//! OrgService: one sealed Persona store per Persona (LLR-f9x3vf), each one
//! Persona's view of one Organisation (SDD-undd7q).
//!
//! It reads no chain: the one operation that judges against the chain takes
//! the Organisation's state as a value (LLR-7en38k). It binds no endpoint and
//! sends and receives nothing: org-io receives a body and its sender and
//! hands them to `check_received`, and sends the messages org-node builds.
//!
//! It writes no file: every operation that changes a store returns at most
//! one `StoreChange` for org-io to write (LLR-ay8nvu). A refusal changes
//! nothing: each change is made on a copy of the store's data, sealed, and
//! only then adopted.
//!
//! Gated on the `app` feature.
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{DevicePublicKey, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, Surname};
use rand_core::{CryptoRng, RngCore};

use crate::chain::OrgState;
use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::invitation::{EndpointAddresses, Invite, InviteId};
use crate::keys::{DeviceSecret, X25519Keypair};
use crate::removal::{KeptAcknowledgement, StoreContent};
use crate::revocation::{RevocationNotice, VerifiedAcknowledgement};
use crate::sequence::SeqGuard;
use crate::store::{
    check_preferred_handles, ensure_within_bound, parse_field, parse_members, ExpectedAdmission, FoundingView,
    KeptAdmissionRequest, MemberSnapshot, OrgRecord, Origin, OutstandingInvite, PersonaRecord, PersonaRecordView,
    PersonaStatus, PersonaStore, PersonaView, ProvisionalChange, ProvisionalFounding, ProvisionalMember, ProvisionalOrganisation,
    ProvisionalOrganisationView, RawMemberSnapshot, RecordView, StoreData, StoreWrite,
};
use crate::types::{ChainAccount, Epoch, OrgPrivateKey, PersonaId, SequenceNumber};
use crate::verify::{check_chain_free, VerifyContext};
use crate::verify_org::{AheadLimit, Judgement, VerificationTime, Verified};
use crate::wire::{
    decode_message, encode_message, peek_admission_request, peek_kind_and_org_id, WireMessage, MAX_MESSAGE,
    ORGANISATION_INDEX, ORG_INFORMATION_INDEX,
};

type Trie = OrgTrie<Blake3Hasher>;

/// A trie mirror rebuilt from persisted member snapshots.
pub(crate) fn trie_from_snapshots(snapshots: &[MemberSnapshot]) -> Result<Trie, OrgNodeError> {
    let leaves = snapshots
        .iter()
        .map(|s| {
            MemberLeaf::new(s.id, s.handle.clone(), s.member_key, s.name.clone(), s.surname.clone(), s.device_keys.clone())
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(OrgNodeError::Trie)?;
    Trie::genesis(leaves).map_err(OrgNodeError::Trie)
}

/// The record a first admission extends, decoded from the snapshot its
/// Organisation-information message carries (REQ-d9g6nt, LLR-j6j95z),
/// refusing a value its type's parse refuses with `member.*` (LLR-7yu2xy)
/// and bytes that do not decode as a snapshot with `record_snapshot`: the
/// bytes come from a peer's message, not from a chain read.
// Public only for the fuzz target `fuzz_first_admission_base`; not API.
#[doc(hidden)]
pub fn first_admission_base(record_snapshot: &[u8]) -> Result<Trie, OrgNodeError> {
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(record_snapshot)
        .map_err(|error| OrgNodeError::InvalidField { field: "record_snapshot", reason: error.to_string() })?;
    trie_from_snapshots(&parse_members(raw)?)
}

/// The members a kept base snapshot encodes, each field parsed by its
/// type (LLR-7yu2xy's `member.*` refusals).
fn decode_record_snapshot(bytes: &[u8]) -> Result<Vec<MemberSnapshot>, OrgNodeError> {
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(bytes)
        .map_err(|error| OrgNodeError::InvalidField { field: "org.kept_base_snapshot", reason: error.to_string() })?;
    parse_members(raw)
}

/// The persisted snapshot of a trie member.
fn snapshot_of(m: &MemberLeaf) -> MemberSnapshot {
    MemberSnapshot {
        id: *m.id(),
        handle: m.handle().clone(),
        name: m.name().clone(),
        surname: m.surname().clone(),
        member_key: *m.p2p_key(),
        device_keys: m.p2p_devices().to_vec(),
    }
}

/// A Persona identifier from its Member-as-a-group key: the first 16 bytes,
/// in hex.
fn persona_id_for(member_key: &PersonPublicKey) -> PersonaId {
    use std::fmt::Write as _;
    PersonaId::new(member_key.as_bytes().iter().take(16).fold(String::new(), |mut hex, byte| {
        // Writing to a `String` cannot fail: its `fmt::Write` returns `Ok`
        // on every write, so there is no error to report.
        let _ = write!(hex, "{byte:02x}");
        hex
    }))
}

/// A fresh `MemberId`: 32 bytes from the caller's cryptographic random
/// source, never derived from a key (REQ-d9g6nt, LLR-4rv4kr).
fn fresh_member_id<R: RngCore + CryptoRng>(rng: &mut R) -> MemberId {
    let mut id = [0u8; 32];
    rng.fill_bytes(&mut id);
    MemberId::new(id)
}

/// The one store change an operation returns (LLR-ay8nvu): the store,
/// sealed again. No variant removes a store: org-io deletes a store's file
/// and keychain item.
#[must_use = "org-io must write the store before reporting success (org-io's write-before-success requirement)"]
#[derive(Debug)]
pub enum StoreChange {
    Sealed(StoreWrite),
}

/// Holds one `PersonaStore` per Persona (LLR-f9x3vf). It holds no chain and
/// no endpoint, so it runs headless on values a test passes.
pub struct OrgService {
    stores: Vec<PersonaStore>,
}

impl OrgService {
    /// A service over `stores`, one per Persona; two stores with one Device
    /// key are refused with `DuplicatePersonaStore` (LLR-f9x3vf).
    pub fn new(stores: Vec<PersonaStore>) -> Result<Self, OrgNodeError> {
        for (index, store) in stores.iter().enumerate() {
            if stores[..index].iter().any(|earlier| earlier.device_key() == store.device_key()) {
                return Err(OrgNodeError::DuplicatePersonaStore);
            }
        }
        Ok(Self { stores })
    }

    // ----------------------------------------------------------
    // Proposing a founding (SDD-hwax8v).
    // ----------------------------------------------------------

    /// Propose a founding (LLR-d4gpxm, LLR-9qavm7, LLR-rjphn4): the name,
    /// surname and Handle are parsed first, in that order (`InvalidField`
    /// `member.name`, `member.surname`, `member.handle`), before any draw;
    /// then a fresh Device seed, a member seed and a MemberId are drawn from
    /// `rng`, the founding record listing the founder alone is built and its
    /// Membership root calculated, and only then the Organisation key pair is
    /// drawn, refused with `DuplicateOrgPublicKey` when its public key equals the
    /// founder's Member-as-a-group key or DevicePublicKey. A new store is
    /// added holding the Persona (`org_id` `None`, `handle` set, `status`
    /// `Proposed`), `org_name` and the provisional founding at epoch 1 with no
    /// proxy; no current record, no chain operation. The secret is returned by
    /// value for org-io's keychain and org-node keeps no copy; a refusal adds
    /// no store and returns no secret.
    pub fn propose_founding<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        name: &str,
        surname: &str,
        handle: &str,
        org_name: String,
    ) -> Result<(DevicePublicKey, DeviceSecret, StoreChange), OrgNodeError> {
        let name = parse_field("member.name", Name::parse(name))?;
        let surname = parse_field("member.surname", Surname::parse(surname))?;
        let handle = parse_field("member.handle", Handle::parse(handle))?;
        let secret = DeviceSecret::generate(rng);
        let member_key_pair = X25519Keypair::generate(rng);
        let member_key = member_key_pair.member_key()?;
        let founder = MemberLeaf::new(
            fresh_member_id(rng),
            handle.clone(),
            member_key,
            name.clone(),
            surname.clone(),
            vec![secret.device_key()],
        )
        .map_err(OrgNodeError::Trie)?;
        let trie = Trie::genesis(vec![founder.clone()]).map_err(OrgNodeError::Trie)?;
        let root = trie.root_hash().map_err(OrgNodeError::Trie)?;
        // REQ-ech45n: drawn for this Organisation alone, after the root, and
        // equal to no key of the founding record (LLR-9qavm7).
        let org_key = X25519Keypair::generate(rng);
        let org_pub_key = org_key.org_public_key()?;
        org_pub_key.ensure_distinct_from(&trie.members())?;
        let persona = PersonaRecord {
            persona_id: persona_id_for(&member_key),
            org_id: None,
            name,
            surname,
            handle: Some(handle),
            member_seed: member_key_pair.member_seed(),
            device_key: secret.device_key(),
            member_id: None,
            status: PersonaStatus::Proposed,
        };
        let founding = ProvisionalFounding {
            epoch: Epoch::new(1),
            members: vec![snapshot_of(&founder)],
            root,
            org_pub_key,
            org_private_key: org_key.org_private_key(),
            proxy: None,
        };
        let data = StoreData { org_name, founding: Some(founding), ..StoreData::new(persona) };
        let change = self.add_store(rng, &secret, data)?;
        Ok((secret.device_key(), secret, change))
    }

    /// Record the founding's pure proxy and its Organisation identifier
    /// (LLR-nx87k5, LLR-qqqg4b): sets the founding's `proxy` and
    /// `persona.org_id`. Refused, changing nothing: a store with no
    /// provisional founding (`NoProvisionalFounding`); a founding holding
    /// another proxy or a Persona bound to another Organisation
    /// (`FoundingProxyAlreadyRecorded`). The same proxy and identifier again
    /// are accepted and the store sealed unchanged in content, so a retry
    /// after a lost write succeeds.
    pub fn record_founding_proxy<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        org_id: OrgId,
        proxy: ChainAccount,
    ) -> Result<StoreChange, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let ((), write) = self.change(index, rng, |data| {
            let founding = data.founding.as_mut().ok_or(OrgNodeError::NoProvisionalFounding)?;
            if founding.proxy.is_some_and(|held| held != proxy) || data.persona.org_id.is_some_and(|held| held != org_id) {
                return Err(OrgNodeError::FoundingProxyAlreadyRecorded);
            }
            founding.proxy = Some(proxy);
            data.persona.org_id = Some(org_id);
            Ok(())
        })?;
        Ok(StoreChange::Sealed(write))
    }

    // ----------------------------------------------------------
    // Proposing a Member (SDD-hwax8v).
    // ----------------------------------------------------------

    /// Propose to join the Organisation of `invite` as a new Persona
    /// (LLR-6acxu9, LLR-rjphn4, LLR-w2uyyh). The name, surname and each
    /// preferred Handle are parsed, in that order (`InvalidField`
    /// `member.name`, `member.surname`, `member.preferred_handles[i]`), and
    /// the Handles checked to be one to three and pairwise distinct
    /// (`member.preferred_handles`); an Organisation a held store is bound
    /// to or expects admission to is refused
    /// (`OrganisationHeldByAnotherPersona`) — all before any draw, adding no
    /// store. Then a fresh Device seed and member seed are drawn for this
    /// Organisation alone, and a new store added holding the Persona
    /// (`org_id` the Invite's, no Handle, `Proposed`), the Invite's
    /// Organisation name, the provisional Member and one expected admission;
    /// no chain operation. The secret is returned by value and org-node
    /// keeps no copy.
    pub fn propose_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        invite: &Invite,
        name: &str,
        surname: &str,
        handles: &[&str],
    ) -> Result<(DevicePublicKey, DeviceSecret, StoreChange), OrgNodeError> {
        const HANDLE_FIELDS: [&str; 3] =
            ["member.preferred_handles[0]", "member.preferred_handles[1]", "member.preferred_handles[2]"];
        let name = parse_field("member.name", Name::parse(name))?;
        let surname = parse_field("member.surname", Surname::parse(surname))?;
        let preferred_handles = handles
            .iter()
            .enumerate()
            .map(|(index, handle)| {
                parse_field(HANDLE_FIELDS.get(index).copied().unwrap_or("member.preferred_handles"), Handle::parse(handle))
            })
            .collect::<Result<Vec<_>, _>>()?;
        check_preferred_handles("member.preferred_handles", &preferred_handles)?;
        let org_id = invite.org_id;
        // `data()` refuses only a removed store (`OrganisationRemoved`),
        // which `bound_org_of` already reads for its Organisation; it holds
        // no record and no expectation (LLR-2d5a7m), so nothing is missed by
        // reading the refusal as "neither".
        let held_elsewhere = self.stores.iter().any(|store| {
            bound_org_of(store) == Some(org_id)
                || store.data().ok().is_some_and(|data| {
                    data.record.as_ref().is_some_and(|record| record.org_id == org_id)
                        || data.expected_admissions.iter().any(|expected| expected.org_id == org_id)
                })
        });
        if held_elsewhere {
            return Err(OrgNodeError::OrganisationHeldByAnotherPersona { org_id });
        }
        let secret = DeviceSecret::generate(rng);
        let member_key_pair = X25519Keypair::generate(rng);
        let member_key = member_key_pair.member_key()?;
        let persona = PersonaRecord {
            persona_id: persona_id_for(&member_key),
            org_id: Some(org_id),
            name: name.clone(),
            surname: surname.clone(),
            handle: None,
            member_seed: member_key_pair.member_seed(),
            device_key: secret.device_key(),
            member_id: None,
            status: PersonaStatus::Proposed,
        };
        let provisional = ProvisionalMember {
            org_id,
            invite_id: invite.invite_id,
            issuer: invite.issuer,
            issuer_addresses: invite.issuer_addresses.clone(),
            name,
            surname,
            preferred_handles,
            member_key,
            device_key: secret.device_key(),
        };
        let data = StoreData {
            org_name: invite.org_name.clone(),
            member: Some(provisional),
            expected_admissions: vec![ExpectedAdmission { org_id }],
            ..StoreData::new(persona)
        };
        let change = self.add_store(rng, &secret, data)?;
        Ok((secret.device_key(), secret, change))
    }

    // ----------------------------------------------------------
    // The receive path (chain-free).
    // ----------------------------------------------------------

    /// The check of a received body, in the store of `device_key`
    /// (LLR-94rzrk): `sender` is the Device the transport authenticated for
    /// it. It takes no chain state. A body over `MAX_MESSAGE` is refused
    /// (`MessageTooLarge`); an admission request is read first with
    /// `peek_admission_request` (LLR-6gkxpg); any other body that does not
    /// decode is `MalformedMessage`. Then the sender rule (LLR-283x6f,
    /// LLR-w7b9kd): in a store with a current record, Organisation
    /// information, a revocation notice and an organisation answer only from
    /// a Device the record lists, an organisation request only from a listed
    /// Device or one held as revoked, an acknowledgement and an admission
    /// request from any Device; anything else is `SenderNotListed` before it
    /// is used. Every refusal returns no `StoreChange`.
    ///
    /// In a store with no current record (a provisional founding or a
    /// provisional Member) a body is decoded no further than its kind and
    /// Organisation identifier unless it is Organisation information, or an
    /// organisation answer, for an Organisation an expected admission names
    /// (LLR-e6w3nr):
    /// `AdmissionNotExpected` for other Organisation information, `OrgNotHeld`
    /// for any other kind. Organisation information is kept as a received
    /// provisional organisation (LLR-76bhw9, LLR-s3vq6d), within the bound of
    /// LLR-m76r4k.
    ///
    /// A revocation notice `revocation::check_notice` passes is kept,
    /// within the same bound, and never acted on at receipt
    /// (`RevocationNoticeKept`, LLR-r4qtqe). An acknowledgement is decided by
    /// `revocation::check_acknowledgement` with `sender` and changes nothing
    /// (`Acknowledged`, LLR-5wg8cu).
    ///
    /// An organisation request is answered with the record and its key to a
    /// listed Device and with its notice to a revoked one, the sender's
    /// address-book entry set (`OrganisationRequest`, LLR-7kdhst,
    /// LLR-j338wq, LLR-h9d65p); an organisation answer is kept only while
    /// behind (`OrganisationAnswerKept`, LLR-kd7u2r; with no record,
    /// LLR-e6w3nr).
    pub fn check_received<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        sender: DevicePublicKey,
        body: &[u8],
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let index = self.store_index(device_key)?;
        if body.len() > MAX_MESSAGE {
            return Err(OrgNodeError::MessageTooLarge { length: body.len() });
        }
        if self.stores[index].data()?.record.is_none() {
            return self.check_without_record(rng, index, sender, body);
        }
        if let Some((org_id, invite_id)) = peek_admission_request(body) {
            return self.check_admission_request(rng, index, sender, org_id, invite_id, body);
        }
        let message = decode_message(body)?;
        let org_id = message.org_id();
        let data = self.stores[index].data()?;
        let Some(record) = &data.record else {
            return Err(OrgNodeError::OrgNotHeld { org_id });
        };
        let listed = record.lists_device(&sender);
        let revoked = data.revoked_devices.iter().any(|revoked| revoked.device_key == sender);
        let acted_on = match &message {
            WireMessage::Acknowledgement(_) | WireMessage::AdmissionRequest { .. } => true,
            WireMessage::OrganisationRequest { .. } => listed || revoked,
            WireMessage::OrgInformation { .. } | WireMessage::Revocation(_) | WireMessage::Organisation { .. } => listed,
        };
        if !acted_on {
            return Err(OrgNodeError::SenderNotListed { org_id });
        }
        match message {
            WireMessage::Acknowledgement(ack) => {
                // LLR-5wg8cu: no listing rule; the sender the transport
                // authenticated is the only key passed.
                let verified = crate::revocation::check_acknowledgement(data, sender, ack)?;
                Ok((Checked::Acknowledged(verified), None))
            }
            WireMessage::Revocation(notice) => {
                // LLR-r4qtqe: kept, under the bound, never acted on at
                // receipt; only `verify` decides a kept notice (LLR-jhy6t6).
                crate::revocation::check_notice(data, &notice)?;
                let ((), write) = self.change(index, rng, |data| {
                    data.kept_revocation_notices.push(notice);
                    ensure_within_bound(data)
                })?;
                Ok((Checked::RevocationNoticeKept, Some(StoreChange::Sealed(write))))
            }
            WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } => {
                let base = trie_from_snapshots(&record.trie_members)?;
                let expected = (record.org_id, Some(record.epoch));
                let last_seq = record.last_seq;
                self.keep_information(rng, index, sender, base, last_seq, expected, envelope, record_snapshot, org_private_key)
            }
            WireMessage::OrganisationRequest { org_id, addresses } => {
                self.answer_organisation_request(rng, index, sender, org_id, addresses)
            }
            WireMessage::Organisation { org_id, epoch, record_snapshot, org_private_key } => {
                self.keep_organisation_answer(rng, index, sender, (org_id, epoch), record_snapshot, org_private_key)
            }
            WireMessage::AdmissionRequest { .. } => Err(OrgNodeError::MalformedMessage),
        }
    }

    /// The organisation-request arm of `check_received`. Who may ask is
    /// LLR-283x6f's rule: a listed Device or one held as revoked. The answer
    /// is built whole or not at all (LLR-xc9c6w): the record snapshot and
    /// its key for a listed requester (LLR-7kdhst), the notice proving its
    /// absence from the current record for a revoked one (LLR-j338wq). Then
    /// the requester's address-book entry is set, sealing only when it
    /// changed (LLR-h9d65p). Every refusal keeps nothing.
    fn answer_organisation_request<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        index: usize,
        sender: DevicePublicKey,
        org_id: OrgId,
        addresses: Vec<String>,
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let data = self.stores[index].data()?;
        let record = match &data.record {
            Some(record) if record.org_id == org_id => record,
            _ => return Err(OrgNodeError::OrgNotHeld { org_id }),
        };
        let listed = record.lists_device(&sender);
        let revoked = if listed {
            None
        } else {
            Some(
                data.revoked_devices
                    .iter()
                    .find(|revoked| revoked.device_key == sender)
                    .ok_or(OrgNodeError::SenderNotListed { org_id })?,
            )
        };
        let addresses = parse_field("request.addresses", EndpointAddresses::parse(addresses))?;
        let body = match revoked {
            None => encode_message(&WireMessage::Organisation {
                org_id,
                epoch: record.epoch,
                record_snapshot: encode_record_snapshot(&record.trie_members)?,
                org_private_key: record.org_private_key.clone(),
            })?,
            Some(revoked) => {
                let proof = trie_from_snapshots(&record.trie_members)?
                    .prove_absent(&revoked.member_id, &sender)
                    .map_err(OrgNodeError::Trie)?;
                let notice = RevocationNotice { org_id, member_id: revoked.member_id, device: sender, proof };
                encode_message(&WireMessage::Revocation(notice))?
            }
        };
        let answer = OutgoingMessage { recipient: sender, body };
        if data.address_book.iter().any(|(key, kept)| *key == sender && *kept == addresses) {
            return Ok((Checked::OrganisationRequest { answer }, None));
        }
        let ((), write) = self.change(index, rng, |data| {
            data.set_address(sender, addresses);
            Ok(())
        })?;
        Ok((Checked::OrganisationRequest { answer }, Some(StoreChange::Sealed(write))))
    }

    /// The organisation-answer arm of `check_received` (LLR-kd7u2r) and of
    /// `check_without_record` (LLR-e6w3nr). With a record, an answer is kept
    /// only from a listed Device (`SenderNotListed`), only while the behind
    /// marker is set (`OrganisationNotRequested`), and only for an epoch
    /// after the record's (`StaleChainState`); with none, only while the
    /// behind marker is set. Its snapshot is decoded with the `member.*`
    /// refusals and its root calculated. It is kept as an `Answer`
    /// provisional organisation with the key derived from the carried one
    /// and no Change set, through the one bound-checked keep (LLR-m76r4k).
    /// An identity already held is `AlreadyHeld`, with no change.
    fn keep_organisation_answer<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        index: usize,
        sender: DevicePublicKey,
        (org_id, epoch): (OrgId, Epoch),
        record_snapshot: Vec<u8>,
        org_private_key: OrgPrivateKey,
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let data = self.stores[index].data()?;
        // With a record: LLR-kd7u2r's three conditions, in its order. With
        // none (a provisional Member's store, which `check_without_record`
        // lets through only for the Organisation its expected admission
        // names): the behind marker alone, from whichever Device (LLR-e6w3nr).
        let base_root = match &data.record {
            Some(record) if record.org_id == org_id => {
                if !record.lists_device(&sender) {
                    return Err(OrgNodeError::SenderNotListed { org_id });
                }
                if data.behind.is_none() {
                    return Err(OrgNodeError::OrganisationNotRequested { org_id });
                }
                if epoch <= record.epoch {
                    return Err(OrgNodeError::StaleChainState { org_id, chain_epoch: epoch, record_epoch: record.epoch });
                }
                Some(record.root_hash)
            }
            Some(_) => return Err(OrgNodeError::OrgNotHeld { org_id }),
            None => {
                if data.behind.is_none() {
                    return Err(OrgNodeError::OrganisationNotRequested { org_id });
                }
                None
            }
        };
        let trie = first_admission_base(&record_snapshot)?;
        let root = trie.root_hash().map_err(OrgNodeError::Trie)?;
        let org_pub_key = org_private_key.x25519_keypair().org_public_key()?;
        if data.holds_provisional_identity(&root, &org_pub_key) {
            return Ok((Checked::AlreadyHeld, None));
        }
        let organisation = ProvisionalOrganisation {
            epoch,
            // An answer carries no Change set; in a store with no record
            // there is no base, and the answer's own root stands in (no
            // adoption reads `base_root` of an `Answer`).
            base_root: base_root.unwrap_or(root),
            members: trie.members().iter().map(snapshot_of).collect(),
            root,
            org_pub_key,
            org_private_key,
            change_set: None,
            base_snapshot: None,
            origin: Origin::Answer { from: sender },
            applied: Vec::new(),
            first_ahead_at: None,
        };
        let ((), write) = self.change(index, rng, |data| {
            data.provisional_organisations.push(organisation);
            ensure_within_bound(data)
        })?;
        Ok((Checked::OrganisationAnswerKept { epoch }, Some(StoreChange::Sealed(write))))
    }

    /// `check_received` in a store with no current record (LLR-e6w3nr): the
    /// body's kind and Organisation identifier are read alone
    /// (`MalformedMessage` if it has neither); only Organisation information
    /// for an Organisation an expected admission names is decoded further,
    /// from whichever Device sent it: its snapshot decoded
    /// (`first_admission_base`, `member.*`), the chain-free checks run
    /// against it, and it is kept as a received provisional organisation,
    /// the expectations untouched. Other Organisation information is
    /// `AdmissionNotExpected`; an organisation answer for an expected
    /// Organisation is kept as `keep_organisation_answer` keeps one, while
    /// behind; every other kind `OrgNotHeld`.
    fn check_without_record<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        index: usize,
        sender: DevicePublicKey,
        body: &[u8],
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let (kind, org_id) = peek_kind_and_org_id(body).ok_or(OrgNodeError::MalformedMessage)?;
        let expected = self.stores[index].data()?.expected_admissions.contains(&ExpectedAdmission { org_id });
        if kind == ORGANISATION_INDEX && expected {
            // The peeked kind is the organisation answer's, so a body that
            // decodes is that kind; any other is refused as malformed.
            let WireMessage::Organisation { org_id, epoch, record_snapshot, org_private_key } = decode_message(body)? else {
                return Err(OrgNodeError::MalformedMessage);
            };
            return self.keep_organisation_answer(rng, index, sender, (org_id, epoch), record_snapshot, org_private_key);
        }
        if kind != ORG_INFORMATION_INDEX {
            return Err(OrgNodeError::OrgNotHeld { org_id });
        }
        if !expected {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
        let WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } = decode_message(body)? else {
            return Err(OrgNodeError::MalformedMessage);
        };
        let base = first_admission_base(&record_snapshot)?;
        let expected = (org_id, None);
        self.keep_information(rng, index, sender, base, SequenceNumber::new(0), expected, envelope, record_snapshot, org_private_key)
    }

    /// Keep received Organisation information as a provisional organisation
    /// (LLR-76bhw9, LLR-s3vq6d, LLR-m76r4k): `check_chain_free` against
    /// `base` (`OrgIdMismatch`, `StaleSeq`, `MalformedDelta`,
    /// `DeltaBaseMismatch`), `expected` naming the Organisation and, when
    /// the store holds a record, its epoch: `SeqNotEpoch` unless the Sequence
    /// number is that epoch plus one; the Change set applied to give the next record and root;
    /// `org_pub_key` derived from the carried key. An identity already held
    /// is `AlreadyHeld`, nothing changed; else the keep is measured by the
    /// bound and `ProvisionalOrganisationKept { epoch }` returned with the
    /// store sealed. No chain state is used.
    #[allow(clippy::too_many_arguments)]
    fn keep_information<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        index: usize,
        sender: DevicePublicKey,
        base: Trie,
        last_seq: SequenceNumber,
        expected: (OrgId, Option<Epoch>),
        envelope: Envelope,
        record_snapshot: Vec<u8>,
        org_private_key: OrgPrivateKey,
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let (expected_org_id, record_epoch) = expected;
        let ctx = VerifyContext { expected_org_id, seq_guard: SeqGuard::from_last_seen(last_seq) };
        check_chain_free(&base, &envelope, &ctx)?;
        let delta = envelope.decode_delta()?;
        if let Some(epoch) = record_epoch {
            if envelope.parent_seq.get() != epoch.get() + 1 {
                return Err(OrgNodeError::SeqNotEpoch { seq: envelope.parent_seq.get(), epoch: epoch.get() + 1 });
            }
        }
        let candidate = base.apply_delta(&delta).map_err(OrgNodeError::Trie)?;
        let root = candidate.root_hash();
        let next = candidate.verify_against(&root).map_err(OrgNodeError::Trie)?;
        let org_pub_key = org_private_key.x25519_keypair().org_public_key()?;
        let data = self.stores[index].data()?;
        if data.holds_provisional_identity(&root, &org_pub_key) {
            return Ok((Checked::AlreadyHeld, None));
        }
        let epoch = Epoch::new(envelope.parent_seq.get());
        let kept = ProvisionalOrganisation {
            epoch,
            base_root: base.root_hash().map_err(OrgNodeError::Trie)?,
            members: next.members().iter().map(snapshot_of).collect(),
            root,
            org_pub_key,
            org_private_key,
            change_set: Some(envelope.delta_bytes),
            base_snapshot: Some(record_snapshot),
            origin: Origin::Received { from: sender },
            applied: Vec::new(),
            first_ahead_at: None,
        };
        let ((), write) = self.change(index, rng, |data| {
            data.provisional_organisations.push(kept);
            ensure_within_bound(data)
        })?;
        Ok((Checked::ProvisionalOrganisationKept { epoch }, Some(StoreChange::Sealed(write))))
    }

    /// An admission request to the store at `index` (LLR-6gkxpg,
    /// LLR-v3av9t): its `(org_id, invite_id)` must be outstanding there, else
    /// `InviteNotOutstanding` before any later field is decoded; then the
    /// body is decoded whole (`MalformedMessage`), its DevicePublicKey must be
    /// `sender` (`AdmissionKeyNotSender`), its preferred Handles one to three
    /// and distinct and its addresses parse (`InvalidField`
    /// `request.preferred_handles`, `request.addresses`). The first request
    /// for the invite is kept with `from: sender` — `Accepted`, the store
    /// sealed; a repeat from `sender` is `AlreadyReceived`, one from another
    /// Device `InviteUsedByAnotherDevice`, neither changing anything.
    fn check_admission_request<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        index: usize,
        sender: DevicePublicKey,
        org_id: OrgId,
        invite_id: InviteId,
        body: &[u8],
    ) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
        let data = self.stores[index].data()?;
        if !data.outstanding_invites.iter().any(|invite| invite.org_id == org_id && invite.invite_id == invite_id) {
            return Err(OrgNodeError::InviteNotOutstanding { org_id, invite_id });
        }
        let WireMessage::AdmissionRequest { name, surname, preferred_handles, member_key, device_key, addresses, .. } =
            decode_message(body)?
        else {
            return Err(OrgNodeError::MalformedMessage);
        };
        if device_key != sender {
            return Err(OrgNodeError::AdmissionKeyNotSender { org_id });
        }
        check_preferred_handles("request.preferred_handles", &preferred_handles)?;
        let addresses = parse_field("request.addresses", EndpointAddresses::parse(addresses))?;
        if let Some(kept) = data.kept_admission_requests.iter().find(|kept| kept.invite_id == invite_id) {
            let outcome = if kept.from == sender {
                AdmissionRequestOutcome::AlreadyReceived
            } else {
                AdmissionRequestOutcome::InviteUsedByAnotherDevice
            };
            return Ok((Checked::AdmissionRequest { outcome }, None));
        }
        let request = KeptAdmissionRequest {
            org_id,
            invite_id,
            from: sender,
            name,
            surname,
            preferred_handles,
            member_key,
            device_key,
            addresses,
        };
        let ((), write) = self.change(index, rng, |data| {
            data.kept_admission_requests.push(request);
            Ok(())
        })?;
        Ok((Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::Accepted }, Some(StoreChange::Sealed(write))))
    }

    // ----------------------------------------------------------
    // Reads.
    // ----------------------------------------------------------

    /// The proxy account the record of `org_id` holds, unchanged and
    /// uninterpreted (LLR-x5asbx); `OrgNotHeld` when no store holds it.
    pub fn proxy_account(&self, org_id: OrgId) -> Result<Option<ChainAccount>, OrgNodeError> {
        let index = self.record_index(org_id).ok_or(OrgNodeError::OrgNotHeld { org_id })?;
        Ok(self.stores[index].data()?.record.as_ref().and_then(|record| record.proxy_account))
    }

    /// The Persona's Member-as-a-group key, from its seed, and its
    /// DevicePublicKey, as the store holds it, and no dialling address
    /// (LLR-437fvx). A removed store keeps no Persona identifier
    /// (LLR-2d5a7m), so `persona_id` names none and the answer is
    /// `PersonaIdNotHeld`, not `OrganisationRemoved`: see `persona_index`.
    pub fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        let persona = &self.stores[self.persona_index(persona_id)?].data()?.persona;
        Ok((persona.member_seed.x25519_keypair().member_key()?, persona.device_key))
    }

    /// One record per Active store, in store order (LLR-f9x3vf). `data()`
    /// refuses only a removed store, and a removed store keeps no Persona
    /// record (LLR-2d5a7m), so there is none to list and the refusal is
    /// skipped. Its acknowledgements still to deliver are read by its
    /// DevicePublicKey through `kept_acknowledgements` (LLR-75sn9g), not
    /// through this list.
    pub fn list_personas(&self) -> Vec<&PersonaRecord> {
        self.stores.iter().filter_map(|store| store.data().ok()).map(|data| &data.persona).collect()
    }

    /// `list_personas`'s records as key-free views (LLR-8ru6mv): copies of
    /// every public field, no member seed; changes nothing.
    pub fn persona_records(&self) -> Vec<PersonaRecordView> {
        self.list_personas().into_iter().map(PersonaRecord::view).collect()
    }

    /// Every current record the stores hold, in store order. A removed
    /// store, the only one `data()` refuses, holds no record (LLR-2d5a7m),
    /// so skipping the refusal leaves none out.
    pub fn list_orgs(&self) -> Vec<&OrgRecord> {
        self.stores.iter().filter_map(|store| store.data().ok()).filter_map(|data| data.record.as_ref()).collect()
    }

    /// The first admissions the stores expect, in store order. A removed
    /// store, the only one `data()` refuses, holds no expectation
    /// (LLR-2d5a7m), so skipping the refusal leaves none out.
    pub fn expected_admissions(&self) -> Vec<ExpectedAdmission> {
        self.stores.iter().filter_map(|store| store.data().ok()).flat_map(|data| data.expected_admissions.iter().copied()).collect()
    }

    /// The store data of `device_key`'s Persona, read only, refusing an
    /// unknown key with `PersonaNotHeld` through the lookup every per-store
    /// operation shares (LLR-f9x3vf), and a store in the removed form with
    /// `OrganisationRemoved` (LLR-myfk8w; test builds only).
    #[cfg(feature = "test-support")]
    pub fn store_data(&self, device_key: &DevicePublicKey) -> Result<&StoreData, OrgNodeError> {
        self.stores[self.store_index(device_key)?].data()
    }

    /// The store content of `device_key` in either form, `None` for an
    /// unknown key (test builds only).
    #[cfg(feature = "test-support")]
    pub fn store_content(&self, device_key: &DevicePublicKey) -> Option<&StoreContent> {
        self.store_index(device_key).ok().map(|index| self.stores[index].content())
    }

    // ----------------------------------------------------------
    // The Invite (SDD-n2x3gb).
    // ----------------------------------------------------------

    /// An Invite from the store of `device_key` (LLR-h9f5c9): a store with
    /// no current record is refused (`OrgNotHeld`, or `FoundingNotWritten`
    /// for a founding not yet written), as are issuer addresses
    /// `EndpointAddresses::parse` refuses (`InvalidField
    /// invite.issuer_addresses`), nothing written; otherwise a fresh 32-byte
    /// `InviteId` is drawn, `OutstandingInvite { org_id, invite_id,
    /// invitee_name }` appended, and the Invite returned with the stored
    /// Organisation name and the Persona's DevicePublicKey as its issuer,
    /// with the store's one change.
    pub fn build_invite<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        invitee_name: String,
        issuer_addresses: Vec<String>,
    ) -> Result<(Invite, StoreChange), OrgNodeError> {
        let index = self.store_index(device_key)?;
        let data = self.stores[index].data()?;
        let org_id = current_record(data)?.org_id;
        let issuer_addresses = parse_field("invite.issuer_addresses", EndpointAddresses::parse(issuer_addresses))?;
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        let invite_id = InviteId::new(bytes);
        let invite = Invite {
            org_name: data.org_name.clone(),
            org_id,
            invitee_name: invitee_name.clone(),
            issuer: data.persona.device_key,
            issuer_addresses,
            invite_id,
        };
        let ((), write) = self.change(index, rng, |data| {
            data.outstanding_invites.push(OutstandingInvite { org_id, invite_id, invitee_name });
            Ok(())
        })?;
        Ok((invite, StoreChange::Sealed(write)))
    }

    /// The admission request of this store's provisional Member, for the
    /// Invite's issuer, carrying the sending Device's own addresses last
    /// (LLR-g32hjn, LLR-h4gwns): an unknown store is `PersonaNotHeld`, a
    /// store with no provisional Member `NoProvisionalMember`, addresses
    /// `EndpointAddresses::parse` refuses `InvalidField request.addresses`.
    /// Changes nothing, so a retry with the same addresses sends the same
    /// bytes.
    pub fn build_admission_request(
        &self,
        device_key: &DevicePublicKey,
        sender_addresses: Vec<String>,
    ) -> Result<OutgoingMessage, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let member = self.stores[index].data()?.member.as_ref().ok_or(OrgNodeError::NoProvisionalMember)?;
        parse_field("request.addresses", EndpointAddresses::parse(sender_addresses.clone()))?;
        let body = encode_message(&WireMessage::AdmissionRequest {
            org_id: member.org_id,
            invite_id: member.invite_id,
            name: member.name.clone(),
            surname: member.surname.clone(),
            preferred_handles: member.preferred_handles.clone(),
            member_key: member.member_key,
            device_key: member.device_key,
            addresses: sender_addresses,
        })?;
        Ok(OutgoingMessage { recipient: member.issuer, body })
    }

    /// One organisation request per Device the current record lists other
    /// than this Persona's, in record order; in a provisional Member's
    /// store, one for the Invite's issuer (LLR-vyank2). The caller's
    /// addresses travel last (LLR-h4gwns); addresses
    /// `EndpointAddresses::parse` refuses are `InvalidField
    /// request.addresses`. A store with neither is `OrgNotHeld`, or
    /// `FoundingNotWritten` for a founding store whose proxy is not yet
    /// recorded. Changes nothing.
    pub fn build_organisation_request(
        &self,
        device_key: &DevicePublicKey,
        sender_addresses: Vec<String>,
    ) -> Result<Vec<OutgoingMessage>, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let data = self.stores[index].data()?;
        let (org_id, recipients): (OrgId, Vec<DevicePublicKey>) = match (&data.record, &data.member) {
            (Some(record), _) => (
                record.org_id,
                record.devices_other_than(data.persona.device_key).collect(),
            ),
            (None, Some(member)) => (member.org_id, vec![member.issuer]),
            (None, None) => {
                return Err(match data.persona.org_id {
                    Some(org_id) => OrgNodeError::OrgNotHeld { org_id },
                    None => OrgNodeError::FoundingNotWritten,
                })
            }
        };
        parse_field("request.addresses", EndpointAddresses::parse(sender_addresses.clone()))?;
        let body = encode_message(&WireMessage::OrganisationRequest { org_id, addresses: sender_addresses })?;
        Ok(recipients.into_iter().map(|recipient| OutgoingMessage { recipient, body: body.clone() }).collect())
    }

    /// Cancel the invite `invite_id`: remove its kept admission request,
    /// its outstanding pair and any `Admit` naming it, in one sealed
    /// change, so a later request carrying it is refused as not
    /// outstanding (LLR-ag8hjd). An invite id with no kept request is
    /// refused with `NoAdmissionRequestKept`, changing nothing.
    pub fn cancel_invite<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        invite_id: InviteId,
    ) -> Result<StoreChange, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let org_id = self.stores[index]
            .data()?
            .kept_request(invite_id)
            .map(|request| request.org_id)
            .ok_or(OrgNodeError::NoAdmissionRequestKept { invite_id })?;
        let ((), write) = self.change(index, rng, |data| {
            data.kept_admission_requests.retain(|request| request.invite_id != invite_id);
            data.outstanding_invites
                .retain(|outstanding| !(outstanding.org_id == org_id && outstanding.invite_id == invite_id));
            data.provisional_changes.retain(|change| !change.admits(invite_id));
            Ok(())
        })?;
        Ok(StoreChange::Sealed(write))
    }

    // ----------------------------------------------------------
    // The stores as org-io's files hold them.
    // ----------------------------------------------------------

    /// Put `store` in place of the held store with its DevicePublicKey, or add it
    /// when none is held: org-io's reload of a Persona from its file. One
    /// store per DevicePublicKey is kept (LLR-f9x3vf); nothing is sealed, no other
    /// store changes and no store is removed. A replaced store's behind
    /// marker, which no file holds, is kept (LLR-zk7fx2).
    pub fn restore_store(&mut self, mut store: PersonaStore) {
        match self.stores.iter().position(|held| held.device_key() == store.device_key()) {
            Some(index) => {
                if let (Ok(held), Ok(restored)) = (self.stores[index].data(), store.data_mut()) {
                    restored.behind = held.behind;
                }
                self.stores[index] = store;
            }
            None => self.stores.push(store),
        }
    }

    /// Remove the store of `device_key` from the in-memory service, returning
    /// whether one was held: org-io's local-IO exception (LLR-9jn3z8). It
    /// touches no file and no keychain item.
    pub fn drop_store(&mut self, device_key: &DevicePublicKey) -> bool {
        let before = self.stores.len();
        self.stores.retain(|held| &held.device_key() != device_key);
        self.stores.len() != before
    }

    /// Verify the store `device_key` names against `chain`, the
    /// Organisation state org-io read at a finalised block (LLR-rr558z):
    /// a founding with no proxy recorded is `FoundingNotWritten`, no state
    /// `OrgNotOnChain`, and otherwise exactly one `Verified`. Against a
    /// state not below the record's epoch, first the received provisional organisations that can never
    /// be adopted are discarded and those one epoch ahead stamped, aged
    /// by `ahead_limit` (LLR-4tsgfh), and the outcome judged on what is
    /// left. `InStep`, `Behind` and `Inconsistent` call `device_secrets`
    /// never and return a store change only when that discard or stamp
    /// changed the store; the behind marker, which is not sealed, changes
    /// too (LLR-qvp7mq, LLR-zk7fx2). After the `Inconsistent`
    /// checks a kept notice `revocation::accept` takes, or a match not
    /// listing the Persona's Device, is its own removal: `Removed`, with
    /// the removed store sealed (LLR-jhy6t6, LLR-5tvn7g).
    pub fn verify<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        chain: Option<OrgState>,
        verified_at: VerificationTime,
        ahead_limit: AheadLimit,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        let index = self.store_index(device_key)?;
        let data = self.stores[index].data()?;
        if data.founding.as_ref().is_some_and(|founding| founding.proxy.is_none()) {
            return Err(OrgNodeError::FoundingNotWritten);
        }
        let chain = chain.ok_or(OrgNodeError::OrgNotOnChain)?;
        let discarded = crate::verify_org::discard_unadoptable(data, &chain, verified_at, ahead_limit);
        let pruned_changed = discarded.is_some();
        let pruned = discarded.unwrap_or_else(|| data.clone());
        let data = &pruned;
        let judgement = crate::verify_org::judge(data, &chain);
        // After the Inconsistent checks, a kept notice accepted against the chain is this
        // Persona's removal (LLR-jhy6t6).
        let inconsistent = matches!(judgement, Judgement::Done(Verified::Inconsistent { .. }));
        if !inconsistent && crate::verify_org::accepted_notice(data, &chain) {
            return self.remove_own(index, rng, &chain, None, verified_at, device_secrets);
        }
        // A match not listing this Persona's Device is its removal (LLR-jhy6t6).
        if let Judgement::Adopt(matched) = judgement {
            let proposal = &data.provisional_organisations[matched];
            let own = data.persona.device_key;
            if !proposal.members.iter().any(|member| member.device_keys.contains(&own)) {
                let members = proposal.members.clone();
                return self.remove_own(index, rng, &chain, Some(members), verified_at, device_secrets);
            }
        }
        let (next, verified) = match judgement {
            Judgement::Done(verified) => {
                let change = if pruned_changed {
                    let write = self.stores[index].seal_data(&pruned, rng)?;
                    let behind = self.stores[index].data()?.behind;
                    *self.stores[index].data_mut()? = StoreData { behind, ..pruned };
                    Some(StoreChange::Sealed(write))
                } else {
                    None
                };
                update_behind_marker(self.stores[index].data_mut()?, &verified);
                return Ok((verified, change));
            }
            Judgement::Adopt(matched) => crate::verify_org::adopt(data, current_record(data)?, matched),
            Judgement::AdoptFirst(matched) => crate::verify_org::adopt_first(data, matched, &chain)?,
            Judgement::AdoptFounding => crate::verify_org::adopt_founding(data)?,
        };
        // Sealed before the outcome is returned (LLR-34c5y3).
        let write = self.stores[index].seal_data(&next, rng)?;
        *self.stores[index].data_mut()? = next;
        Ok((verified, Some(StoreChange::Sealed(write))))
    }

    // ----------------------------------------------------------
    // The Admin's provisional changes (SDD-hwax8v).
    // ----------------------------------------------------------

    /// Record the admission of the request kept for `invite_id`
    /// (LLR-bwtx3h): the first of its preferred Handles, in ranked order,
    /// that neither a member of the current record nor another `Admit`
    /// holds, else the parsed `chosen`; appends `Admit { invite_id,
    /// handle }` and changes nothing else (LLR-b2hgd4). Refused, recording
    /// nothing, as LLR-mq7n6u states.
    pub fn record_admission<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        invite_id: InviteId,
        chosen: Option<&str>,
    ) -> Result<StoreChange, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let ((), write) = self.change(index, rng, |data| {
            let handle = admission_handle(data, invite_id, chosen)?;
            data.provisional_changes.push(ProvisionalChange::Admit { invite_id, handle });
            ensure_within_bound(data)
        })?;
        Ok(StoreChange::Sealed(write))
    }

    /// Record the revocation of `member_id` (LLR-f3hru6): appends
    /// `Revoke { member_id }`; refuses a MemberId the record does not
    /// hold, one a `Revoke` already names, a store with no record, the
    /// behind marker set and the bound, recording nothing.
    pub fn record_revocation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        member_id: MemberId,
    ) -> Result<StoreChange, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let ((), write) = self.change(index, rng, |data| {
            let record = current_record(data)?;
            ensure_not_behind(data, record)?;
            if !record.trie_members.iter().any(|member| member.id == member_id) {
                return Err(OrgNodeError::MemberNotHeld { member_id });
            }
            let named = |change: &ProvisionalChange| {
                matches!(change, ProvisionalChange::Revoke { member_id: revoked } if *revoked == member_id)
            };
            if data.provisional_changes.iter().any(named) {
                return Err(OrgNodeError::RevocationAlreadyRecorded { member_id });
            }
            data.provisional_changes.push(ProvisionalChange::Revoke { member_id });
            ensure_within_bound(data)
        })?;
        Ok(StoreChange::Sealed(write))
    }

    /// Calculate the next epoch from every provisional change, in recorded
    /// order, and keep it as one `Own` provisional organisation for the
    /// record's epoch plus one (LLR-va2yqz); the changes stay held and the
    /// record is unchanged. Refused, keeping nothing, as LLR-n6ssue and LLR-qghs95
    /// state. The proposal is built before `change`, whose seal draws from
    /// the same random source after it.
    pub fn propose_updated_organisation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
    ) -> Result<StoreChange, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let proposal = proposal_of(self.stores[index].data()?, rng)?;
        let ((), write) = self.change(index, rng, |data| {
            data.provisional_organisations.push(proposal);
            ensure_within_bound(data)
        })?;
        Ok(StoreChange::Sealed(write))
    }

    // ----------------------------------------------------------
    // The messages for the current epoch (SDD-qzv5mn).
    // ----------------------------------------------------------

    /// The messages of the epoch the store holds as current (LLR-degg8t):
    /// refused for a store with no record (`OrgNotHeld`); none when the
    /// record keeps no Change set; otherwise one Organisation-information
    /// body (the kept Change set under the record's epoch, the kept base
    /// snapshot, the record's private key) for each Device the record
    /// lists other than this Persona's, in record order, then one
    /// revocation notice for each Device the base lists and the record
    /// does not. Changes nothing; equal stores give equal results.
    pub fn messages_for_current_epoch(&self, device_key: &DevicePublicKey) -> Result<Vec<OutgoingMessage>, OrgNodeError> {
        let data = self.stores[self.store_index(device_key)?].data()?;
        let record = current_record(data)?;
        let (Some(change_set), Some(base_snapshot)) = (&record.kept_change_set, &record.kept_base_snapshot) else {
            return Ok(Vec::new());
        };
        let information = encode_message(&WireMessage::OrgInformation {
            envelope: Envelope { org_id: record.org_id, parent_seq: record.last_seq, delta_bytes: change_set.clone() },
            record_snapshot: base_snapshot.clone(),
            org_private_key: record.org_private_key.clone(),
        })?;
        let own = data.persona.device_key;
        let mut outgoing: Vec<OutgoingMessage> = record
            .devices_other_than(own)
            .map(|recipient| OutgoingMessage { recipient, body: information.clone() })
            .collect();
        let base = OrgRecord { trie_members: decode_record_snapshot(base_snapshot)?, ..record.clone() };
        let trie = trie_from_snapshots(&record.trie_members)?;
        for notice in crate::revocation::notices_for(&base, record, &trie)? {
            let recipient = notice.device;
            outgoing.push(OutgoingMessage { recipient, body: encode_message(&WireMessage::Revocation(notice))? });
        }
        Ok(outgoing)
    }

    // ----------------------------------------------------------
    // Read-only views (LLR-8ru6mv): no private key, no seed, no secret.
    // ----------------------------------------------------------

    /// The store's provisional changes, in recorded order, by reference
    /// (LLR-8ru6mv).
    pub fn provisional_changes(&self, device_key: &DevicePublicKey) -> Result<&[ProvisionalChange], OrgNodeError> {
        Ok(&self.stores[self.store_index(device_key)?].data()?.provisional_changes)
    }

    /// Each provisional organisation's public values, with no private key
    /// (LLR-8ru6mv).
    pub fn provisional_organisations(
        &self,
        device_key: &DevicePublicKey,
    ) -> Result<Vec<ProvisionalOrganisationView>, OrgNodeError> {
        let data = self.stores[self.store_index(device_key)?].data()?;
        let record_epoch = data.record.as_ref().map(|record| record.epoch);
        Ok(data
            .provisional_organisations
            .iter()
            .map(|held| {
                let base_epoch = Epoch::new(held.epoch.get().saturating_sub(1));
                ProvisionalOrganisationView {
                    epoch: held.epoch,
                    base_epoch,
                    root: held.root,
                    org_pub_key: held.org_pub_key,
                    origin: held.origin,
                    current_proposal: matches!(held.origin, Origin::Own)
                        && record_epoch == Some(base_epoch)
                        && held.applied == data.provisional_changes,
                }
            })
            .collect())
    }

    /// The kept admission requests, in kept order, each with the invitee
    /// name of the outstanding invite whose identifier it carries
    /// (LLR-8ru6mv). A request is kept only for an outstanding invite and
    /// both go together (LLR-6gkxpg, LLR-rzqs2u, LLR-ag8hjd), so the empty
    /// name is never returned in practice.
    pub fn admission_requests(
        &self,
        device_key: &DevicePublicKey,
    ) -> Result<Vec<(&KeptAdmissionRequest, &str)>, OrgNodeError> {
        let data = self.stores[self.store_index(device_key)?].data()?;
        Ok(data
            .kept_admission_requests
            .iter()
            .map(|request| {
                let name = data
                    .outstanding_invites
                    .iter()
                    .find(|invite| invite.org_id == request.org_id && invite.invite_id == request.invite_id)
                    .map_or("", |invite| invite.invitee_name.as_str());
                (request, name)
            })
            .collect())
    }

    /// A copy of the public values of `device_key`'s store (LLR-8ru6mv):
    /// the Persona's DevicePublicKey and binding, the Organisation name, the
    /// current record's values, the provisional founding's with its proxy,
    /// and the Organisation a provisional Member asked to join. It copies
    /// no seed, secret or private key and changes nothing; an unknown key
    /// is `PersonaNotHeld`.
    pub fn persona_view(&self, device_key: &DevicePublicKey) -> Result<PersonaView, OrgNodeError> {
        let data = self.stores[self.store_index(device_key)?].data()?;
        Ok(PersonaView {
            device_key: data.persona.device_key,
            org_id: data.persona.org_id,
            org_name: data.org_name.clone(),
            record: data.record.as_ref().map(|record| RecordView {
                org_id: record.org_id,
                root_hash: record.root_hash,
                org_pub_key: record.org_pub_key,
                epoch: record.epoch,
                members: record.trie_members.clone(),
            }),
            founding: data.founding.as_ref().map(|founding| FoundingView {
                epoch: founding.epoch,
                root: founding.root,
                org_pub_key: founding.org_pub_key,
                proxy: founding.proxy,
            }),
            member_of: data.member.as_ref().map(|member| member.org_id),
        })
    }

    /// The addresses this store keeps for `peer` (LLR-pgy2rb): its
    /// address-book entry; else a kept request's from it; else, in a
    /// provisional Member's store whose Invite it issued, the Invite's
    /// issuer addresses; else none. Changes nothing.
    pub fn addresses_for(
        &self,
        device_key: &DevicePublicKey,
        peer: DevicePublicKey,
    ) -> Result<Option<&EndpointAddresses>, OrgNodeError> {
        let data = self.stores[self.store_index(device_key)?].data()?;
        if let Some((_, addresses)) = data.address_book.iter().find(|(device, _)| *device == peer) {
            return Ok(Some(addresses));
        }
        if let Some(request) = data.kept_admission_requests.iter().find(|request| request.from == peer) {
            return Ok(Some(&request.addresses));
        }
        Ok(data.member.as_ref().filter(|member| member.issuer == peer).map(|member| &member.issuer_addresses))
    }

    // ----------------------------------------------------------
    // The kept acknowledgements of a removed store (SDD-z4m57w).
    // ----------------------------------------------------------

    /// The recipient `recipient` accepted its acknowledgement: remove it, with that key and its addresses,
    /// and return how many are still kept and the store sealed (LLR-85kves). An Active store, or a
    /// recipient no kept acknowledgement names, is refused with `AcknowledgementNotKept`. The answer
    /// itself is not judged here; org-io received it.
    pub fn acknowledgement_accepted<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        device_key: &DevicePublicKey,
        recipient: DevicePublicKey,
    ) -> Result<(usize, StoreChange), OrgNodeError> {
        let index = self.store_index(device_key)?;
        let StoreContent::Removed(removed) = self.stores[index].content() else {
            return Err(OrgNodeError::AcknowledgementNotKept { recipient: Box::new(recipient) });
        };
        let mut next = removed.clone();
        if !next.remove_acknowledgement(&recipient) {
            return Err(OrgNodeError::AcknowledgementNotKept { recipient: Box::new(recipient) });
        }
        let left = next.acknowledgements.len();
        let content = StoreContent::Removed(next);
        let write = self.stores[index].seal_content(&content, rng)?;
        self.stores[index].replace_content(content);
        Ok((left, StoreChange::Sealed(write)))
    }

    /// The acknowledgements a removed store keeps, as messages to their recipients with the recipients'
    /// addresses; `None` for an Active store (LLR-75sn9g). Changes nothing.
    pub fn kept_acknowledgements(&self, device_key: &DevicePublicKey) -> Result<Option<KeptAcknowledgements>, OrgNodeError> {
        let index = self.store_index(device_key)?;
        let StoreContent::Removed(removed) = self.stores[index].content() else {
            return Ok(None);
        };
        let acknowledgements = removed
            .acknowledgements
            .iter()
            .map(|kept| {
                let body = encode_message(&WireMessage::Acknowledgement(kept.ack.clone()))?;
                Ok((OutgoingMessage { recipient: kept.recipient, body }, kept.recipient_addresses.clone()))
            })
            .collect::<Result<Vec<_>, OrgNodeError>>()?;
        Ok(Some(KeptAcknowledgements { org_id: removed.org_id, verified_at: removed.verified_at, acknowledgements }))
    }

    // ----------------------------------------------------------
    // Private helpers.
    // ----------------------------------------------------------

    /// The index of the store of `device_key`'s Persona, else
    /// `PersonaNotHeld` (LLR-f9x3vf): the lookup of every per-store operation.
    fn store_index(&self, device_key: &DevicePublicKey) -> Result<usize, OrgNodeError> {
        self.stores
            .iter()
            .position(|store| &store.device_key() == device_key)
            .ok_or_else(|| OrgNodeError::PersonaNotHeld { device_key: Box::new(*device_key) })
    }

    /// The index of `persona_id`'s store, else `PersonaIdNotHeld`. A removed
    /// store, the only one `data()` refuses, keeps no Persona identifier
    /// (LLR-2d5a7m), so no `persona_id` can name it and skipping the
    /// refusal finds every store that does.
    fn persona_index(&self, persona_id: &PersonaId) -> Result<usize, OrgNodeError> {
        self.stores
            .iter()
            .position(|store| store.data().ok().is_some_and(|data| &data.persona.persona_id == persona_id))
            .ok_or_else(|| OrgNodeError::PersonaIdNotHeld { persona_id: persona_id.clone() })
    }

    /// The index of the store holding the current record of `org_id`. A
    /// removed store, the only one `data()` refuses, holds no record
    /// (LLR-2d5a7m), so skipping the refusal finds every store that does.
    fn record_index(&self, org_id: OrgId) -> Option<usize> {
        self.stores.iter().position(|store| {
            store.data().ok().is_some_and(|data| data.record.as_ref().is_some_and(|record| record.org_id == org_id))
        })
    }

    /// Add the new store `data` keyed from `secret`, sealed with a fresh
    /// nonce: refused (`DuplicatePersonaStore`) when a store with its Device
    /// key is held, adding nothing (LLR-f9x3vf).
    fn add_store<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        secret: &DeviceSecret,
        data: StoreData,
    ) -> Result<StoreChange, OrgNodeError> {
        if self.store_index(&data.persona.device_key).is_ok() {
            return Err(OrgNodeError::DuplicatePersonaStore);
        }
        let store = PersonaStore::create(secret, data)?;
        let write = store.seal(rng)?;
        self.stores.push(store);
        Ok(StoreChange::Sealed(write))
    }

    /// Replace the store at `index` with its removed form, sealed, and return
    /// it as the one `StoreChange::Sealed` (LLR-2d5a7m). No other store is
    /// read or changed (LLR-f9x3vf).
    fn seal_removal<R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        acknowledgements: Vec<KeptAcknowledgement>,
        verified_at: VerificationTime,
    ) -> Result<StoreChange, OrgNodeError> {
        let data = self.stores[index].data()?.clone();
        let content = StoreContent::Removed(data.into_removed(acknowledgements, verified_at)?);
        let write = self.stores[index].seal_content(&content, rng)?;
        self.stores[index].replace_content(content);
        Ok(StoreChange::Sealed(write))
    }

    /// This Persona's own removal (LLR-jhy6t6, LLR-5tvn7g, LLR-vq5d5n): choose the recipients among
    /// the record's other Devices and, when there is at least one, call `device_secrets` once for the
    /// Persona's key, sign one acknowledgement and keep it for each recipient with its address-book
    /// entry. Then seal the removed store. With no secret it refuses `DeviceSecretNotSupplied`, changing
    /// nothing. Every secret is dropped before return.
    fn remove_own<R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        chain: &OrgState,
        matched: Option<Vec<MemberSnapshot>>,
        verified_at: VerificationTime,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        let data = self.stores[index].data()?;
        let record = data.record.as_ref().ok_or(OrgNodeError::OrgNotOnChain)?;
        let org_id = record.org_id;
        let own = data.persona.device_key;
        let recipients = crate::removal::recipients(&record.trie_members, &own, matched.as_deref());
        let mut acknowledgements = Vec::with_capacity(recipients.len());
        if !recipients.is_empty() {
            let secrets = device_secrets(&[own]);
            let secret = secrets
                .iter()
                .find(|secret| secret.device_key() == own)
                .ok_or(OrgNodeError::DeviceSecretNotSupplied { org_id })?;
            let ack = crate::revocation::sign_acknowledgement(&data.persona, org_id, chain, secret)?;
            drop(secrets);
            for recipient in recipients {
                let recipient_addresses =
                    data.address_book.iter().find(|(key, _)| *key == recipient).map(|(_, addresses)| addresses.clone());
                acknowledgements.push(KeptAcknowledgement { ack: ack.clone(), recipient, recipient_addresses });
            }
        }
        let kept = acknowledgements.len();
        let change = self.seal_removal(index, rng, acknowledgements, verified_at)?;
        Ok((Verified::Removed { org_id, acknowledgements_kept: kept }, Some(change)))
    }

    /// Make `change` to a copy of the data of the store at `index`, seal
    /// the copy, and only then adopt it: a refusal, of the change or of the
    /// seal, leaves the store as it was and returns no write.
    fn change<T, R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        change: impl FnOnce(&mut StoreData) -> Result<T, OrgNodeError>,
    ) -> Result<(T, StoreWrite), OrgNodeError> {
        let mut data = self.stores[index].data()?.clone();
        let result = change(&mut data)?;
        let write = self.stores[index].seal_data(&data, rng)?;
        *self.stores[index].data_mut()? = data;
        Ok((result, write))
    }
}

/// The Organisation `store` is bound to, read from its content in either
/// form: a removed store stays bound to its Organisation (LLR-w2uyyh).
fn bound_org_of(store: &PersonaStore) -> Option<OrgId> {
    match store.content() {
        StoreContent::Active(data) => data.persona.org_id,
        StoreContent::Removed(removed) => Some(removed.org_id),
    }
}

/// The behind marker after `verified` (LLR-zk7fx2): set on `Behind`,
/// cleared on `InStep`, left as it was otherwise.
fn update_behind_marker(data: &mut StoreData, verified: &Verified) {
    match verified {
        Verified::Behind { chain_epoch } => data.behind = Some(*chain_epoch),
        Verified::InStep => data.behind = None,
        Verified::Inconsistent { .. } | Verified::Adopted { .. } | Verified::Removed { .. } => {}
    }
}

/// The current record of `data`'s store, else `OrgNotHeld` naming the
/// Persona's Organisation, or `FoundingNotWritten` for a founding store whose
/// Organisation identifier is not yet recorded.
pub(crate) fn current_record(data: &StoreData) -> Result<&OrgRecord, OrgNodeError> {
    match (&data.record, data.persona.org_id) {
        (Some(record), _) => Ok(record),
        (None, Some(org_id)) => Err(OrgNodeError::OrgNotHeld { org_id }),
        (None, None) => Err(OrgNodeError::FoundingNotWritten),
    }
}

/// Refuses while the behind marker is set (LLR-zk7fx2).
fn ensure_not_behind(data: &StoreData, record: &OrgRecord) -> Result<(), OrgNodeError> {
    match data.behind {
        Some(chain_epoch) => Err(OrgNodeError::OrganisationBehind { org_id: record.org_id, chain_epoch }),
        None => Ok(()),
    }
}

/// The record snapshot a base snapshot keeps: the members, postcard-encoded.
fn encode_record_snapshot(snapshots: &[MemberSnapshot]) -> Result<Vec<u8>, OrgNodeError> {
    postcard::to_allocvec(snapshots).map_err(|error| OrgNodeError::Chain(format!("record snapshot encode: {error}")))
}

/// The `Own` provisional organisation of the store's current record and
/// provisional changes (LLR-va2yqz): the trie rebuilt, every change
/// applied in recorded order (an `Admit` under a MemberId drawn from
/// `rng` with the kept request's values), the root calculated once, the
/// Change set encoded, then one Organisation key pair drawn and checked
/// distinct from the record's key and every key of the result (LLR-qghs95).
fn proposal_of<R: RngCore + CryptoRng>(data: &StoreData, rng: &mut R) -> Result<ProvisionalOrganisation, OrgNodeError> {
    let record = current_record(data)?;
    ensure_not_behind(data, record)?;
    if data.provisional_changes.is_empty() {
        return Err(OrgNodeError::NoProvisionalChange);
    }
    let mut trie = trie_from_snapshots(&record.trie_members)?;
    for change in &data.provisional_changes {
        trie = match change {
            ProvisionalChange::Admit { invite_id, handle } => {
                let request =
                    data.kept_request(*invite_id).ok_or(OrgNodeError::NoAdmissionRequestKept { invite_id: *invite_id })?;
                let leaf = MemberLeaf::new(
                    fresh_member_id(rng),
                    handle.clone(),
                    request.member_key,
                    request.name.clone(),
                    request.surname.clone(),
                    vec![request.device_key],
                )
                .map_err(OrgNodeError::Trie)?;
                trie.add_member(leaf).map_err(OrgNodeError::Trie)?
            }
            ProvisionalChange::Revoke { member_id } => trie.delete_member(member_id).map_err(OrgNodeError::Trie)?,
        };
    }
    let (next, delta) = trie.recalculate().map_err(OrgNodeError::Trie)?;
    let root = next.root_hash().map_err(OrgNodeError::Trie)?;
    let epoch = Epoch::new(record.epoch.get() + 1);
    let change_set = Envelope::build(record.org_id, SequenceNumber::new(epoch.get()), &delta)?.delta_bytes;
    let members = next.members();
    let org_key = X25519Keypair::generate(rng);
    let org_pub_key = org_key.org_public_key()?;
    if org_pub_key == record.org_pub_key {
        return Err(OrgNodeError::DuplicateOrgPublicKey);
    }
    org_pub_key.ensure_distinct_from(&members)?;
    Ok(ProvisionalOrganisation {
        epoch,
        base_root: record.root_hash,
        members: members.iter().map(snapshot_of).collect(),
        root,
        org_pub_key,
        org_private_key: org_key.org_private_key(),
        change_set: Some(change_set),
        base_snapshot: Some(encode_record_snapshot(&record.trie_members)?),
        origin: Origin::Own,
        applied: data.provisional_changes.clone(),
        first_ahead_at: None,
    })
}

/// The Handle an admission of the request kept for `invite_id` is given
/// (LLR-bwtx3h), or the refusal LLR-mq7n6u states.
fn admission_handle(data: &StoreData, invite_id: InviteId, chosen: Option<&str>) -> Result<Handle, OrgNodeError> {
    let record = current_record(data)?;
    ensure_not_behind(data, record)?;
    let request = data.kept_request(invite_id).ok_or(OrgNodeError::NoAdmissionRequestKept { invite_id })?;
    if data.provisional_changes.iter().any(|change| change.admits(invite_id)) {
        return Err(OrgNodeError::AdmissionAlreadyRecorded { invite_id });
    }
    let taken = |handle: &Handle| {
        record.trie_members.iter().any(|member| &member.handle == handle)
            || data
                .provisional_changes
                .iter()
                .any(|change| matches!(change, ProvisionalChange::Admit { handle: given, .. } if given == handle))
    };
    if let Some(free) = request.preferred_handles.iter().find(|handle| !taken(handle)) {
        return Ok(free.clone());
    }
    let chosen = parse_field("admission.handle", Handle::parse(chosen.ok_or(OrgNodeError::NoFreeHandle)?))?;
    if taken(&chosen) {
        return Err(OrgNodeError::HandleTaken { handle: chosen });
    }
    Ok(chosen)
}

/// One recipient's body (LLR-rsjs8e): the Device it goes to and the encoded
/// Wire message. Not `Clone`: an Organisation-information body carries the
/// Organisation private key (REQ-y7tsft); its `Debug` renders the recipient
/// and the body's length only.
pub struct OutgoingMessage {
    pub recipient: DevicePublicKey,
    pub body: Vec<u8>,
}

/// What a removed store keeps for org-io's delivery (LLR-75sn9g): the Organisation, the verification time
/// for the 7-day deadline, and each acknowledgement as a message to its recipient with that recipient's
/// kept addresses, in kept order. Not `Clone`.
#[derive(Debug)]
pub struct KeptAcknowledgements {
    pub org_id: OrgId,
    pub verified_at: VerificationTime,
    pub acknowledgements: Vec<(OutgoingMessage, Option<EndpointAddresses>)>,
}

impl std::fmt::Debug for OutgoingMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OutgoingMessage")
            .field("recipient", &self.recipient)
            .field("body_len", &self.body.len())
            .finish()
    }
}

/// What `check_received` decided (LLR-94rzrk). Every outcome but a kept
/// one, an accepted admission request and an organisation request that
/// changed the sender's addresses comes with no `StoreChange`.
// One value per received message, moved once to org-io.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Checked {
    /// Organisation information kept as a received provisional organisation
    /// for `epoch` (LLR-76bhw9).
    ProvisionalOrganisationKept { epoch: Epoch },
    /// Organisation information whose identity is already held (LLR-s3vq6d).
    AlreadyHeld,
    /// A revocation notice kept for verification (LLR-r4qtqe).
    RevocationNoticeKept,
    /// An admission request, with what was decided (LLR-6gkxpg).
    AdmissionRequest { outcome: AdmissionRequestOutcome },
    /// An organisation request answered (LLR-7kdhst, LLR-j338wq).
    OrganisationRequest { answer: OutgoingMessage },
    /// An organisation answer kept for `epoch` (LLR-kd7u2r).
    OrganisationAnswerKept { epoch: Epoch },
    /// An acknowledgement `check_acknowledgement` accepted (LLR-5wg8cu).
    Acknowledged(VerifiedAcknowledgement),
}

/// What an admission request decided (LLR-6gkxpg).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionRequestOutcome {
    /// The first request for its invite: kept.
    Accepted,
    /// A repeat from the Device whose request is kept: nothing changed.
    AlreadyReceived,
    /// A request for an invite whose kept request came from another Device:
    /// nothing changed.
    InviteUsedByAnotherDevice,
}
