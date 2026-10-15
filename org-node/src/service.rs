//! OrgService: composes the Persona stores into the five user stories. See spec §6 and plan docs/superpowers/plans/2026-06-16-ods-phase-2-4-tauri-shell.md Task 2.
//!
//! It reads no chain (ruling B, change `worktree-org-io-create`): every
//! operation that judges against the chain takes the Organisation's state as
//! an `Option<OrgState>` argument, which org-io reads, and is synchronous. A
//! receive is one path in two steps, on values (LLR-zetn7a, LLR-6qjmhf):
//! `check_received(sender, body)` decodes the body and runs every check that
//! needs no chain state, deciding an acknowledgement there or naming the
//! Organisation whose state it needs; `commit_checked` decides the checked
//! message against that state. It binds no endpoint and sends and receives
//! nothing: org-io receives the body and its sender, and
//! sends the bodies a commit returns.
//!
//! It writes no file: it holds one sealed store per Persona (LLR-rdda86), and every operation
//! that changes one returns that store's `StoreWrite` (or, when the store's
//! Persona is forgotten, `StoreChange::Forgotten`) for org-io to write
//! (LLR-bvt5up). A refusal changes nothing: each change is made on a copy of
//! the store's data, sealed, and only then adopted.
//!
//! Gated on the `app` feature.
use org_members::delta::Delta;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{DevicePublicKey, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, RootHash, Surname};
use rand_core::{CryptoRng, RngCore};

use crate::chain::OrgState;
use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::invitation::{Invite, InviteId, InviteReply};
use crate::keys::{DeviceSecret, X25519Keypair};
use crate::revocation::{Acknowledgement, RevocationNotice, VerifiedAcknowledgement};
use crate::sequence::SeqGuard;
use crate::store::{
    ExpectedAdmission, MemberSnapshot, OrgRecord, PersonaRecord, PersonaStatus, PersonaStore, ProvisionalChange,
    ProvisionalUpdate, RawMemberSnapshot, StoreData, StoreWrite,
};
pub use crate::store::ProvisionalTarget;
use crate::types::{ChainAccount, Epoch, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};
use crate::verify::{VerifiedUpdate, VerifyContext, verify_envelope_against_chain};
use crate::wire::{WireMessage, decode_message, encode_message};

type Trie = OrgTrie<Blake3Hasher>;

// ============================================================
// Helper: rebuild a trie mirror from persisted MemberSnapshots.
// ============================================================

fn trie_from_snapshots(snapshots: &[MemberSnapshot]) -> Result<Trie, OrgNodeError> {
    let leaves = snapshots
        .iter()
        .map(|s| {
            MemberLeaf::new(s.id, s.handle.clone(), s.member_key, s.name.clone(), s.surname.clone(), s.device_keys.clone())
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(OrgNodeError::Trie)?;
    Trie::genesis(leaves).map_err(OrgNodeError::Trie)
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

/// The record a first admission extends, decoded from the snapshot its
/// Organisation-information message carries (REQ-d9g6nt, LLR-j6j95z).
// Public only for the fuzz target `fuzz_first_admission_base`; not API.
#[doc(hidden)]
pub fn first_admission_base(record_snapshot: &[u8]) -> Result<Trie, OrgNodeError> {
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(record_snapshot)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot decode: {e}")))?;
    let snaps = raw.into_iter().map(MemberSnapshot::try_from).collect::<Result<Vec<_>, _>>()?;
    trie_from_snapshots(&snaps)
}

/// Encode the record a committed update extends, as an
/// Organisation-information message's `record_snapshot`;
/// `first_admission_base` decodes it.
fn encode_record_snapshot(snapshots: &[MemberSnapshot]) -> Result<Vec<u8>, OrgNodeError> {
    postcard::to_allocvec(snapshots)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot encode: {e}")))
}

/// The person an admission adds, as parsed values the app built from the
/// Invite reply it parsed (LLR-kkj64b).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Joiner {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
}

/// What an operation that may forget a Persona did to its store
/// (LLR-bvt5up, LLR-pba7yu): sealed it again, or forgot it with its
/// Persona, so that org-io removes that Persona's file.
#[must_use = "org-io must write or remove the store before reporting success (org-io's write-before-success requirement)"]
#[derive(Debug)]
pub enum StoreChange {
    Sealed(StoreWrite),
    Forgotten { device_key: DevicePublicKey },
}

// ============================================================
// OrgService — the composition root.
// ============================================================

/// Composes one `PersonaStore` per Persona into the five org-node user
/// stories. It holds no chain and no endpoint: the
/// chain-judging operations take the state as a value, a receive takes the
/// body and its sender as values, and a commit returns the bodies to send,
/// so the full composition runs headless on values a test passes.
pub struct OrgService {
    stores: Vec<PersonaStore>,
}

impl OrgService {
    /// A service over `stores`, one per Persona; two stores with one Device
    /// key are refused with `DuplicatePersonaStore` (LLR-rdda86).
    pub fn new(stores: Vec<PersonaStore>) -> Result<Self, OrgNodeError> {
        for (index, store) in stores.iter().enumerate() {
            if stores[..index].iter().any(|earlier| earlier.device_key() == store.device_key()) {
                return Err(OrgNodeError::DuplicatePersonaStore);
            }
        }
        Ok(Self { stores })
    }

    // ----------------------------------------------------------
    // Story precursor: create a persona (no chain, no network).
    // ----------------------------------------------------------

    /// Generate a new Persona with a fresh X25519 member seed and a fresh
    /// device secret, open a new store for it keyed from that secret, and
    /// return the Persona's id, the secret, by value, for org-io to keep in
    /// the keychain, and the new store's write; org-node keeps no copy of the
    /// secret (LLR-e5kqhg).
    pub fn create_persona<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: Handle,
        name: Name,
        surname: Surname,
    ) -> Result<(PersonaId, DeviceSecret, StoreWrite), OrgNodeError> {
        let member_kp = X25519Keypair::generate(rng);
        let secret = DeviceSecret::generate(rng);
        // Derive a unique persona_id from the Member-as-a-group key.
        let persona_id = persona_id_for(&member_kp.member_key()?);
        let record = PersonaRecord {
            persona_id: persona_id.clone(),
            org_id: None,
            handle,
            name,
            surname,
            member_seed: member_kp.member_seed(),
            device_key: secret.device_key(),
            member_id: None,
            status: PersonaStatus::Proposed,
        };
        let store = PersonaStore::create(&secret, record)?;
        if self.stores.iter().any(|held| held.device_key() == store.device_key()) {
            return Err(OrgNodeError::DuplicatePersonaStore);
        }
        let write = store.seal(rng)?;
        self.stores.push(store);
        Ok((persona_id, secret, write))
    }

    // ----------------------------------------------------------
    // Story 1: create_organisation builds the genesis update; commit_genesis
    // records it once the chain carries it.
    // ----------------------------------------------------------

    /// Build the genesis provisional update for `persona_id` and keep it: a
    /// fresh Organisation key pair whose private key the update holds, no
    /// chain operation, no record, no binding (LLR-s6qnht, LLR-qjz3q4,
    /// LLR-68yd3j, LLR-sj7cd5). A Persona already bound to an Organisation
    /// is refused (LLR-6z5xya).
    pub fn create_organisation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<(ProvisionalUpdate, StoreWrite), OrgNodeError> {
        self.ensure_unbound(persona_id)?;
        let index = self.persona_index(persona_id)?;
        let persona = &self.stores[index].data().persona;
        let founder = MemberLeaf::new(
            fresh_member_id(rng),
            persona.handle.clone(),
            persona.member_seed.x25519_keypair().member_key()?,
            persona.name.clone(),
            persona.surname.clone(),
            vec![persona.device_key],
        )
        .map_err(OrgNodeError::Trie)?;
        let trie = Trie::genesis(vec![founder.clone()]).map_err(OrgNodeError::Trie)?;
        // REQ-ech45n: drawn for this Organisation alone, its public key equal
        // to no key of the genesis record (LLR-sj7cd5).
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        org_pub_key.ensure_distinct_from(&trie.members())?;
        let update = ProvisionalUpdate {
            org_id: None,
            persona_id: persona_id.clone(),
            base_root: None,
            resulting_root: trie.root_hash().map_err(OrgNodeError::Trie)?,
            // Genesis produces epoch 1 on the contract (Decision 16).
            seq: SequenceNumber::new(1),
            org_pub_key,
            change: ProvisionalChange::Genesis {
                members: vec![snapshot_of(&founder)],
                org_private_key: org_kp.org_private_key(),
            },
        };
        let ((), write) = self.change(index, rng, |data| data.insert_provisional(update.clone()))?;
        Ok((update, write))
    }

    /// Commit the genesis update `persona_id` built, once the chain carries
    /// its root and key at epoch 1, against `chain_state`, the state org-io
    /// read for `org_id` (`None`: `OrgNotOnChain`): select the update,
    /// require the epoch to be the update's Sequence number and the members
    /// to rebuild the root, then create the record with the chain's values,
    /// mark 1, the update's private key and `proxy_account`, consume the
    /// update with the Persona's other genesis updates (LLR-pk6mhv) and bind
    /// the Persona, which must be unbound, with the MemberId of the genesis
    /// leaf listing its Device (LLR-hby4jr) (LLR-wzqqg9, LLR-qjz3q4,
    /// LLR-w3fhhg, LLR-dzte8x, LLR-eyc4ud). Any refusal changes nothing and
    /// returns no write (LLR-ewkg85); no endpoint is bound (LLR-4tcxsu).
    pub fn commit_genesis<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
        proxy_account: ChainAccount,
        chain_state: Option<OrgState>,
    ) -> Result<(ReceiveOutcome, StoreWrite), OrgNodeError> {
        // One Persona, one Organisation (REQ-yp75u9, LLR-eyc4ud).
        self.ensure_unbound(persona_id)?;
        let index = self.persona_index(persona_id)?;
        let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;
        let update = self
            .genesis_updates(persona_id)
            .find(|u| u.resulting_root == state.root_hash && u.org_pub_key == state.org_pub_key)
            .cloned()
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        // A chain already past epoch 1 — another signatory updated the
        // Organisation first — is refused too: the record then comes from an
        // update another Member sends (owner ruling 2026-10-06).
        if state.epoch.get() != update.seq.get() {
            return Err(OrgNodeError::SeqNotEpoch { seq: update.seq.get(), epoch: state.epoch.get() });
        }
        let ProvisionalChange::Genesis { members, org_private_key } = update.change.clone() else {
            return Err(OrgNodeError::NoProvisionalUpdate);
        };
        if trie_from_snapshots(&members)?.root_hash().map_err(OrgNodeError::Trie)? != state.root_hash {
            return Err(OrgNodeError::RootMismatch);
        }
        // The founder's MemberId is the genesis leaf listing this Persona's
        // Device; the Persona is bound with it (LLR-hby4jr, PR-eqs4fs), and
        // an update with no such leaf is refused (LLR-wzqqg9).
        // `create_organisation` builds that leaf from this Persona, so a
        // genesis update without it is, like a non-genesis change above, no
        // update this Persona can commit.
        let device = self.stores[index].device_key();
        let founder_id = members
            .iter()
            .find(|member| member.device_keys.contains(&device))
            .map(|member| member.id)
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        // The founding consumes its update and drops the Persona's other
        // genesis updates, each with its private key (LLR-pk6mhv): they are
        // all in this Persona's store.
        let founding = ProvisionalTarget::Genesis(persona_id.clone());
        let ((), write) = self.change(index, rng, |data| {
            data.orgs.push(OrgRecord {
                org_id,
                root_hash: state.root_hash,
                org_pub_key: state.org_pub_key,
                epoch: state.epoch,
                last_seq: update.seq,
                trie_members: members,
                proxy_account: Some(proxy_account),
                org_private_key,
                kept_change_set: None,
            });
            data.provisional_updates.retain(|provisional| !founding.names(provisional));
            discard_orphans(data, org_id, state.root_hash);
            bind_persona(data, org_id, founder_id);
            Ok(())
        })?;
        // The founder is admitted to its own Organisation.
        let outcome = ReceiveOutcome {
            org_id,
            epoch: state.epoch,
            root: state.root_hash,
            kind: ReceiveKind::Admitted,
            acknowledgements: Vec::new(),
        };
        Ok((outcome, write))
    }

    /// Remove the one provisional update for `target` whose resulting root
    /// is `resulting_root` and whose Organisation public key is
    /// `org_pub_key`, with the Organisation private key it holds, and return
    /// the write of the store that held it; refuse one not stored with
    /// `NoProvisionalUpdate`, changing and returning nothing (LLR-7cmp38).
    pub fn discard_provisional<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        target: ProvisionalTarget,
        resulting_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<StoreWrite, OrgNodeError> {
        let names = |provisional: &ProvisionalUpdate| {
            provisional.resulting_root == resulting_root
                && provisional.org_pub_key == org_pub_key
                && target.names(provisional)
        };
        let index = self
            .stores
            .iter()
            .position(|store| store.data().provisional_updates.iter().any(names))
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        let ((), write) = self.change(index, rng, |data| {
            let position =
                data.provisional_updates.iter().position(names).ok_or(OrgNodeError::NoProvisionalUpdate)?;
            data.provisional_updates.remove(position);
            Ok(())
        })?;
        Ok(write)
    }

    /// The stored provisional updates for `org_id` (LLR-nvn3wk).
    pub fn provisional_updates(&self, org_id: OrgId) -> Vec<ProvisionalUpdate> {
        self.stores
            .iter()
            .flat_map(|store| store.data().provisional_updates.iter())
            .filter(|provisional| provisional.org_id == Some(org_id))
            .cloned()
            .collect()
    }

    /// Whether the store of the Persona bound to `org_id` holds a
    /// provisional update for `org_id` built on `base_root` with
    /// `resulting_root` and `org_pub_key`, compared by reference: no update,
    /// and no private key it holds, is copied (LLR-gwk4nk).
    pub fn holds_provisional(
        &self,
        org_id: OrgId,
        base_root: RootHash,
        resulting_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> bool {
        self.bound_index(org_id).is_some_and(|index| {
            self.stores[index].data().provisional_updates.iter().any(|provisional| {
                provisional.org_id == Some(org_id)
                    && provisional.base_root == Some(base_root)
                    && provisional.resulting_root == resulting_root
                    && provisional.org_pub_key == org_pub_key
            })
        })
    }

    /// The genesis updates `persona_id` built (LLR-nvn3wk).
    pub fn genesis_provisional_updates(&self, persona_id: &PersonaId) -> Vec<ProvisionalUpdate> {
        self.genesis_updates(persona_id).cloned().collect()
    }

    fn genesis_updates<'a>(&'a self, persona_id: &'a PersonaId) -> impl Iterator<Item = &'a ProvisionalUpdate> + 'a {
        let target = ProvisionalTarget::Genesis(persona_id.clone());
        self.stores
            .iter()
            .filter(move |store| &store.data().persona.persona_id == persona_id)
            .flat_map(|store| store.data().provisional_updates.iter())
            .filter(move |provisional| target.names(provisional))
    }

    /// The proxy account the record holds, unchanged and uninterpreted
    /// (LLR-3v5nu9).
    pub fn proxy_account(&self, org_id: OrgId) -> Result<Option<ChainAccount>, OrgNodeError> {
        Ok(self.find_org(org_id)?.proxy_account)
    }

    /// The Persona's Member-as-a-group key, from its seed, and its
    /// DevicePublicKey, as the store holds it, and no dialling address
    /// (LLR-437fvx).
    pub fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        let p = self.find_persona(persona_id)?;
        Ok((p.member_seed.x25519_keypair().member_key()?, p.device_key))
    }

    // ----------------------------------------------------------
    // Story 3: admit_member builds the admission; commit_update records it
    // once the chain carries it and returns every recipient's body.
    // ----------------------------------------------------------

    /// Build the admission of `joiner` into `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-rb8r65,
    /// LLR-vdyu65). The new Member's id is drawn, so a re-admission with the
    /// same keys gets a new one (REQ-d9g6nt).
    pub fn admit_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &Joiner,
    ) -> Result<(ProvisionalUpdate, StoreWrite), OrgNodeError> {
        let index = self.record_index(org_id)?;
        let rec = self.find_org(org_id)?.clone();
        let leaf = MemberLeaf::new(
            fresh_member_id(rng),
            joiner.handle.clone(),
            joiner.member_key,
            joiner.name.clone(),
            joiner.surname.clone(),
            vec![joiner.device_key],
        )
        .map_err(OrgNodeError::Trie)?;
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .add_member(leaf)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(index, rng, &rec, &new_trie, &delta)
    }

    /// Keep a Change set built on `rec` as a provisional update of the store
    /// at `index`: its Sequence number is the epoch it produces, the record's
    /// plus one (LLR-ghja3x, REQ-txvtm9, Decision 16). Its root calculated,
    /// the batch ends and a fresh Organisation key pair is drawn for it
    /// (REQ-stx9v3, LLR-e2b7gv): distinct from the record's current key and
    /// from every key of the resulting record, else `DuplicateKey`, nothing
    /// kept or written. The update holds the private key; the record's keys
    /// are not changed. A refusal by the bound writes nothing (LLR-jq7qh7).
    fn keep_change_set<R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        rec: &OrgRecord,
        new_trie: &Trie,
        delta: &Delta,
    ) -> Result<(ProvisionalUpdate, StoreWrite), OrgNodeError> {
        let seq = SequenceNumber::new(rec.epoch.get() + 1);
        let resulting_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        if org_pub_key == rec.org_pub_key {
            return Err(OrgNodeError::Trie(org_members::OrgMembersError::DuplicateKey));
        }
        org_pub_key.ensure_distinct_from(&new_trie.members())?;
        let update = ProvisionalUpdate {
            org_id: Some(rec.org_id),
            persona_id: self.stores[index].data().persona.persona_id.clone(),
            base_root: Some(rec.root_hash),
            resulting_root,
            seq,
            org_pub_key,
            change: ProvisionalChange::ChangeSet {
                change_set: Envelope::build(rec.org_id, seq, delta)?.delta_bytes,
                org_private_key: org_kp.org_private_key(),
            },
        };
        let ((), write) = self.change(index, rng, |data| data.insert_provisional(update.clone()))?;
        Ok((update, write))
    }

    /// Commit the node's own provisional update for `org_id` once the chain
    /// carries its root and key, against `chain_state`, the state org-io read
    /// back after its write (`None`: `OrgNotOnChain`), by the checks a
    /// received update passes, taking the update's key pair into the record
    /// (LLR-cmdrp9, LLR-6s785x), and return every recipient's body, built
    /// with that key (LLR-gdfqn9).
    /// A refusal changes nothing and returns no store change (LLR-ewkg85);
    /// nothing is bound or sent (LLR-4tcxsu). A commit that removes the
    /// Persona bound to the Organisation forgets it instead, through the one
    /// removal step, signing with the secrets `device_secrets` yields for the
    /// Device keys it names — called on that path alone — and returns the
    /// acknowledgements it signed and `StoreChange::Forgotten` (LLR-b27jr6,
    /// LLR-jd82t6, LLR-uw7nmv).
    pub fn commit_update<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        chain_state: Option<OrgState>,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(CommitOutcome, StoreChange), OrgNodeError> {
        let index = self.record_index(org_id)?;
        let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;
        let data = self.stores[index].data();
        let update = data.held_update_for(org_id, &state).cloned().ok_or(OrgNodeError::NoProvisionalUpdate)?;
        let committed = commit_step(data, org_id, &update, &state, device_secrets)?;
        let change = self.adopt(index, rng, committed.successor)?;
        Ok((committed.outcome, change))
    }

    /// The chain-free half of a received revocation notice (LLR-pt32fx):
    /// `check_notice` first (LLR-38e2kn, LLR-r7zm39); then `sender` must be
    /// listed in the record (LLR-kzgjz8). A refusal returns no checked
    /// message, so no chain state is asked for.
    fn check_revocation(&self, sender: &DevicePublicKey, notice: RevocationNotice) -> Result<CheckedKind, OrgNodeError> {
        let index = self.notice_index(&notice)?;
        sender_listed(find_record(self.stores[index].data(), notice.org_id)?, notice.org_id, sender)?;
        Ok(CheckedKind::Revocation { notice })
    }

    /// The marks of `org_id`'s record as a store holds it now, `None` when
    /// no store does (PR-erskh4).
    fn record_marks(&self, org_id: OrgId) -> Option<RecordMarks> {
        self.find_org(org_id).ok().map(|record| RecordMarks {
            epoch: record.epoch,
            root: record.root_hash,
            last_seq: record.last_seq,
        })
    }

    /// The checked message for `kind`, with the marks of the record its
    /// check read (PR-erskh4).
    fn checked(&self, kind: CheckedKind) -> CheckedMessage {
        let record_at_check = self.record_marks(kind.org_id());
        CheckedMessage { kind, record_at_check }
    }

    /// The store whose Persona a notice is for (`revocation::notice_store`,
    /// LLR-r7zm39, LLR-38e2kn).
    fn notice_index(&self, notice: &RevocationNotice) -> Result<usize, OrgNodeError> {
        crate::revocation::notice_store(self.stores.iter().map(PersonaStore::data), notice)
    }

    /// The commit half of a received revocation notice: `revocation::accept`
    /// against `chain_state` (LLR-tx8ruv). A refusal changes nothing. On
    /// acceptance the successor store is adopted; returns the state the
    /// notice was accepted against, the acknowledgements and the store
    /// change.
    fn accept_revocation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        notice: &RevocationNotice,
        chain_state: Option<OrgState>,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(OrgState, Vec<Acknowledgement>, StoreChange), OrgNodeError> {
        let index = self.notice_index(notice)?;
        let accepted = crate::revocation::accept(self.stores[index].data(), notice, chain_state, device_secrets)?;
        // `accept` succeeds only with a chain state.
        let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;
        let change = self.adopt(index, rng, Successor::Removed(accepted.store))?;
        Ok((state, accepted.acknowledgements, change))
    }

    /// The chain-free half of received Organisation information, from the
    /// held record (`existing`) or, for a first admission (`None`), from the
    /// snapshot the message carries: the Change set must pass every check
    /// that needs no chain (LLR-9ew26y, RC-6a2dke, RC-e5atck).
    fn check_org_information(
        envelope: Envelope,
        record_snapshot: &[u8],
        carried_key: OrgPrivateKey,
        existing: Option<&OrgRecord>,
        expectation: Option<ExpectedAdmission>,
    ) -> Result<CheckedKind, OrgNodeError> {
        let (local_trie, last_seq, last_epoch) = match existing {
            Some(rec) => (trie_from_snapshots(&rec.trie_members)?, rec.last_seq, rec.epoch),
            // The record a first admission extends: from the snapshot its
            // Organisation-information message carries (REQ-d9g6nt,
            // LLR-j6j95z).
            None => (first_admission_base(record_snapshot)?, SequenceNumber::new(0), Epoch::new(0)),
        };
        let ctx = VerifyContext {
            expected_org_id: envelope.org_id,
            seq_guard: SeqGuard::from_last_seen(last_seq),
            last_committed_epoch: last_epoch,
        };
        crate::verify::check_chain_free(&local_trie, &envelope, &ctx)?;
        Ok(CheckedKind::OrgInformation { envelope, carried_key, local_trie, ctx, expectation })
    }

    // ----------------------------------------------------------
    // Story 4: receive — one path, by values: the check with no chain
    // state, then the commit against the state org-io read (ruling B).
    // ----------------------------------------------------------

    /// The check of a received body (LLR-zetn7a): `sender` is the Device the
    /// transport authenticated for it, passed in by org-io. The body is
    /// decoded first (`MalformedMessage`, before any record, expectation or
    /// chain is consulted: LLR-xn5pwc, REQ-c29s93). Then, with no chain
    /// state: an acknowledgement is decided here against the store alone,
    /// only from the Device it names (LLR-3aysup, LLR-5azhry:
    /// `Checked::Acknowledged`); a revocation passes `check_notice`
    /// (LLR-38e2kn, LLR-r7zm39) and the sender rule (LLR-kzgjz8);
    /// Organisation information about a held Organisation is acted on only
    /// from a Device the record lists (LLR-2r2fha), a first admission from
    /// any sender (REQ-xa6smf) but only when a store expects it (LLR-s8xp7m),
    /// and either must decode its snapshot and pass `check_chain_free`
    /// (LLR-9ew26y). A refusal returns no `CheckedMessage`, so no chain
    /// state is asked for. Writes nothing.
    pub fn check_received(&self, sender: DevicePublicKey, body: &[u8]) -> Result<Checked, OrgNodeError> {
        // What the message is (LLR-js9dsu). A revocation is decided by its
        // proof — an unheld Organisation refused before the expectations are
        // consulted (LLR-38e2kn) — and an acknowledgement by the store alone
        // (LLR-pt32fx).
        let (envelope, record_snapshot, carried_key) = match decode_message(body)? {
            WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } => {
                (envelope, record_snapshot, org_private_key)
            }
            WireMessage::Revocation(notice) => {
                return self.check_revocation(&sender, notice).map(|kind| Checked::ReadChain(self.checked(kind)));
            }
            WireMessage::Acknowledgement(ack) => {
                return self.check_acknowledgement(sender, ack).map(Checked::Acknowledged);
            }
        };
        let org_id = envelope.org_id;

        // The record this message extends. No Invite is required (REQ-xa6smf,
        // owner ruling 2026-10-05). For a held Organisation the sender must
        // be listed in the record, before anything is decoded (LLR-2r2fha);
        // a first admission is accepted from any sender (owner amendment of
        // 2026-10-07).
        let existing = self.find_org(org_id).ok();
        if let Some(record) = existing {
            sender_listed(record, org_id, &sender)?;
        }
        // A first admission is read only when a store expects one to this
        // Organisation — before its snapshot is decoded (LLR-s8xp7m,
        // RC-2ferct).
        let expectation = match existing {
            Some(_) => None,
            None => {
                let expectation = ExpectedAdmission { org_id };
                if !self.expected_admissions().contains(&expectation) {
                    return Err(OrgNodeError::AdmissionNotExpected { org_id });
                }
                Some(expectation)
            }
        };
        Self::check_org_information(envelope, &record_snapshot, carried_key, existing, expectation)
            .map(|kind| Checked::ReadChain(self.checked(kind)))
    }

    /// `revocation::check_acknowledgement` against the store holding the
    /// Organisation's record; `AcknowledgementNotHeld` when none does
    /// (LLR-5azhry).
    fn check_acknowledgement(
        &self,
        sender: DevicePublicKey,
        ack: Acknowledgement,
    ) -> Result<VerifiedAcknowledgement, OrgNodeError> {
        let org_id = ack.org_id;
        let index = self.record_index(org_id).map_err(|_| OrgNodeError::AcknowledgementNotHeld { org_id })?;
        crate::revocation::check_acknowledgement(self.stores[index].data(), sender, ack)
    }

    /// The commit of a checked message (LLR-6qjmhf), against `chain_state`,
    /// the state org-io read once for `checked.org_id()` (`None`: no state,
    /// `OrgNotOnChain`). A revocation is decided by `revocation::accept`
    /// (LLR-tx8ruv: `Revoked`); Organisation information is verified against
    /// the state and its carried key checked (LLR-ba2ejp), then committed as
    /// a first admission (`Admitted`, LLR-3f5h7b) or an update (`Updated`),
    /// or — when the committed record no longer lists this node — through
    /// the one removal step (`Removed`, LLR-b27jr6). Only the revocation and
    /// the removal call `device_secrets`, and only after every check passed
    /// (LLR-uw7nmv). A refusal changes nothing and returns no store change.
    ///
    /// First of all, the record the check read must be the record held
    /// now: a check whose record's epoch, root or sequence guard moved, or
    /// whose record appeared or went, since the check is refused with
    /// `RecordMovedSinceCheck`, so a local commit that fell between the check
    /// and the commit is never judged with the values it replaced (PR-erskh4).
    pub fn commit_checked<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        checked: CheckedMessage,
        chain_state: Option<OrgState>,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(ReceiveOutcome, StoreChange), OrgNodeError> {
        let checked_org = checked.org_id();
        if self.record_marks(checked_org) != checked.record_at_check {
            return Err(OrgNodeError::RecordMovedSinceCheck { org_id: checked_org });
        }
        let (envelope, carried_key, local_trie, ctx, expectation) = match checked.kind {
            CheckedKind::Revocation { notice } => {
                let (state, acknowledgements, change) =
                    self.accept_revocation(rng, &notice, chain_state, device_secrets)?;
                let outcome = ReceiveOutcome {
                    org_id: notice.org_id,
                    epoch: state.epoch,
                    root: state.root_hash,
                    kind: ReceiveKind::Revoked,
                    acknowledgements,
                };
                return Ok((outcome, change));
            }
            CheckedKind::OrgInformation { envelope, carried_key, local_trie, ctx, expectation } => {
                (envelope, carried_key, local_trie, ctx, expectation)
            }
        };
        let org_id = envelope.org_id;
        // What the Change set must reach (RC-6a2dke, RC-e5atck).
        let chain_state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;
        let verified = verify_envelope_against_chain(&local_trie, &envelope, &ctx, Some(chain_state))?;
        // The key Organisation information carries must be the private half
        // of the chain's key (LLR-ba2ejp, RC-9cefcn) — checked before the
        // own-Persona rule and before anything is written.
        check_carried_key(org_id, &carried_key, &chain_state)?;
        let new_root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
        let outcome = |kind, acknowledgements| ReceiveOutcome {
            org_id,
            epoch: verified.epoch,
            root: new_root,
            kind,
            acknowledgements,
        };

        // The check passed a first admission, and only a first admission,
        // with the expectation it matched.
        if let Some(expectation) = expectation {
            // A first admission commits only into a store that declared the
            // expectation and whose unbound Persona's Device the record lists
            // (LLR-3f5h7b, REQ-kt877x, LLR-rdda86); the refusal writes nothing.
            let members = verified.trie.members();
            let ours = self.stores.iter().enumerate().find_map(|(index, store)| {
                let data = store.data();
                if data.persona.org_id.is_some() || !data.expected_admissions.contains(&expectation) {
                    return None;
                }
                members
                    .iter()
                    .find(|member| member.has_p2p_device(&data.persona.device_key))
                    .map(|member| (index, *member.id()))
            });
            let Some((index, member_id)) = ours else {
                return Err(OrgNodeError::AdmissionNotOurs { org_id });
            };
            let ((), write) = self.change(index, rng, |data| {
                data.orgs.push(OrgRecord {
                    org_id,
                    root_hash: new_root,
                    // The chain's key, never a value from the Wire message
                    // (LLR-xq9nrq).
                    org_pub_key: chain_state.org_pub_key,
                    epoch: verified.epoch,
                    last_seq: verified.seq_guard.last_seen(),
                    trie_members: members.iter().map(snapshot_of).collect(),
                    // A first-admission record: only `commit_genesis` keeps a
                    // proxy account (LLR-3v5nu9).
                    proxy_account: None,
                    org_private_key: carried_key,
                    // The Change set of the update that admitted this node
                    // (REQ-uv3v5w, LLR-d9778a).
                    kept_change_set: Some(envelope.delta_bytes.clone()),
                });
                // Clear the expectation the committed first admission
                // matched, and only that one (LLR-q8emds).
                data.expected_admissions.retain(|e| *e != expectation);
                discard_orphans(data, org_id, new_root);
                bind_persona(data, org_id, member_id);
                Ok(())
            })?;
            return Ok((outcome(ReceiveKind::Admitted, Vec::new()), StoreChange::Sealed(write)));
        }

        let index = self.record_index(org_id)?;
        if !still_member(self.stores[index].data(), org_id, &verified.trie) {
            // The node's own removal, on this path as on every other, through
            // the one removal step (LLR-b27jr6, LLR-jd82t6): nothing is
            // committed, nothing bound.
            let (successor, acknowledgements) =
                removal_step(self.stores[index].data(), org_id, &chain_state, device_secrets)?;
            let change = self.adopt(index, rng, Successor::Removed(successor))?;
            return Ok((outcome(ReceiveKind::Removed, acknowledgements), change));
        }
        let ((), write) = self.change(index, rng, |data| {
            commit_received(data, org_id, &verified, &envelope, carried_key, &chain_state)?;
            // The Persona marked Active is the store's, whose Device the
            // verified record lists (LLR-e5c9ud, LLR-eyc4ud).
            let members = verified.trie.members();
            let member_id =
                members.iter().find(|member| member.has_p2p_device(&data.persona.device_key)).map(|member| *member.id());
            if let Some(member_id) = member_id {
                bind_persona(data, org_id, member_id);
            }
            Ok(())
        })?;
        Ok((outcome(ReceiveKind::Updated, Vec::new()), StoreChange::Sealed(write)))
    }

    // ----------------------------------------------------------
    // Story 5: revoke_member builds the removal; commit_update records it
    // once the chain carries it and returns every recipient's body.
    // ----------------------------------------------------------

    /// Build the removal of `member_id` from `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-6dc598).
    pub fn revoke_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: MemberId,
    ) -> Result<(ProvisionalUpdate, StoreWrite), OrgNodeError> {
        let index = self.record_index(org_id)?;
        let rec = self.find_org(org_id)?.clone();
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .delete_member(&member_id)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(index, rng, &rec, &new_trie, &delta)
    }

    // ----------------------------------------------------------
    // Query helpers.
    // ----------------------------------------------------------

    /// One record per store, in store order (LLR-rdda86).
    pub fn list_personas(&self) -> Vec<&PersonaRecord> {
        self.stores.iter().map(|store| &store.data().persona).collect()
    }

    /// Every Organisation record the stores hold, in store order.
    pub fn list_orgs(&self) -> Vec<&OrgRecord> {
        self.stores.iter().flat_map(|store| store.data().orgs.iter()).collect()
    }

    /// The store data of `persona_id`, read only, for tests that call the
    /// pure functions (test builds only).
    #[cfg(feature = "test-support")]
    pub fn store_data(&self, persona_id: &PersonaId) -> Option<&StoreData> {
        self.stores.iter().map(PersonaStore::data).find(|data| &data.persona.persona_id == persona_id)
    }

    /// Reconcile `org_id`'s record with `chain_state`, the state org-io read
    /// for it: refuse an unheld Organisation with `OrgNotHeld` before the
    /// state is looked at, whatever it is, then `None` with `OrgNotOnChain`,
    /// then call `reconcile::reconcile` over the store holding the record,
    /// with `device_secrets` called only by a commit that removes this node,
    /// and adopt a successor, returning its store change (LLR-gr8x3r,
    /// LLR-fm38ww). org-io's reconcile calls it.
    pub fn reconcile<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        chain_state: Option<OrgState>,
        device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
    ) -> Result<(crate::reconcile::Reconciled, Option<StoreChange>), OrgNodeError> {
        use crate::reconcile::Reconciled;
        // Held check first, whatever state is given.
        let index = self.record_index(org_id).map_err(|_| OrgNodeError::OrgNotHeld { org_id })?;
        let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;
        let reconciled = crate::reconcile::reconcile(self.stores[index].data(), org_id, state, device_secrets)?;
        let change = match &reconciled {
            Reconciled::Committed { store, .. } => Some(self.adopt(index, rng, Successor::Kept(store.clone()))?),
            Reconciled::Removed { store, .. } => Some(self.adopt(index, rng, Successor::Removed(store.clone()))?),
            Reconciled::InStep | Reconciled::Behind { .. } => None,
        };
        Ok((reconciled, change))
    }

    /// Record, in `persona_id`'s store, that the app expects a first
    /// admission to `org_id`, at most once, and return that store's write
    /// (LLR-9zfnmb). An Organisation another Persona of this node holds or
    /// expects is refused, writing nothing (LLR-yexf5m).
    pub fn expect_admission<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
    ) -> Result<StoreWrite, OrgNodeError> {
        let index = self.persona_index(persona_id)?;
        self.ensure_no_other_membership(persona_id, org_id)?;
        self.record_expectation(index, rng, org_id)
    }

    // ----------------------------------------------------------
    // The Invite and the Invite reply (SDD-xshv5b).
    // ----------------------------------------------------------

    /// An Invite for `org_id` from the Persona bound to it, carrying that
    /// Persona's Device public key and a fresh 32-byte identifier, which its
    /// store keeps as outstanding; returns the Invite and that store's write
    /// (LLR-nr8df3). An Organisation no Persona is bound to is refused
    /// (`OrgNotOnChain`), writing nothing.
    pub fn build_invite<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        org_name: &str,
        invitee_name: &str,
    ) -> Result<(Invite, StoreWrite), OrgNodeError> {
        let index = self.bound_index(org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        let invite_id = InviteId::new(bytes);
        let ((), write) = self.change(index, rng, |data| {
            data.outstanding_invites.push((org_id, invite_id));
            Ok(())
        })?;
        let invite = Invite {
            org_name: org_name.into(),
            org_id,
            invitee_name: invitee_name.into(),
            inviter_device_keys: vec![self.stores[index].device_key()],
            invite_id,
        };
        Ok((invite, write))
    }

    /// The reply to `invite` from `persona_id`, carrying the Invite's
    /// Organisation and identifier and the Persona's member key, Device
    /// public key, handle, name and surname; records the expectation of that
    /// Organisation in the Persona's store at most once and returns that
    /// store's write (LLR-7q9g9z). Refused, writing nothing and building no
    /// reply: an unknown Persona; a Persona bound to an Organisation
    /// (`PersonaAlreadyBound`); an Organisation another Persona of this node
    /// holds or expects (`OrganisationHeldByAnotherPersona`, LLR-yexf5m).
    pub fn build_reply<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        invite: &Invite,
        persona_id: &PersonaId,
    ) -> Result<(InviteReply, StoreWrite), OrgNodeError> {
        let index = self.persona_index(persona_id)?;
        self.ensure_unbound(persona_id)?;
        self.ensure_no_other_membership(persona_id, invite.org_id)?;
        let persona = &self.stores[index].data().persona;
        let reply = InviteReply {
            org_id: invite.org_id,
            invite_id: invite.invite_id,
            member_key: persona.member_seed.x25519_keypair().member_key()?,
            device_key: persona.device_key,
            handle: persona.handle.clone(),
            name: persona.name.clone(),
            surname: persona.surname.clone(),
        };
        let write = self.record_expectation(index, rng, invite.org_id)?;
        Ok((reply, write))
    }

    /// Accept `reply` only for `selected` (`ReplyForAnotherOrganisation`)
    /// and only when its (Organisation, invite identifier) pair is
    /// outstanding in a store (`InviteNotOutstanding`); an identifier
    /// outstanding for another Organisation is not (LLR-d55brn).
    pub fn check_reply(&self, reply: &InviteReply, selected: OrgId) -> Result<(), OrgNodeError> {
        if reply.org_id != selected {
            return Err(OrgNodeError::ReplyForAnotherOrganisation);
        }
        self.outstanding_index(reply.org_id, reply.invite_id).map(|_| ())
    }

    /// Remove exactly the pair (`org_id`, `invite_id`) from the store that
    /// holds it and return that store's write; refused
    /// (`InviteNotOutstanding`), writing nothing, when no store does
    /// (LLR-d55brn).
    pub fn settle<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        invite_id: InviteId,
    ) -> Result<StoreWrite, OrgNodeError> {
        let index = self.outstanding_index(org_id, invite_id)?;
        let ((), write) = self.change(index, rng, |data| {
            data.outstanding_invites.retain(|pair| *pair != (org_id, invite_id));
            Ok(())
        })?;
        Ok(write)
    }

    /// The first admissions the stores declared they expect, in store order.
    pub fn expected_admissions(&self) -> Vec<ExpectedAdmission> {
        self.stores.iter().flat_map(|store| store.data().expected_admissions.iter().copied()).collect()
    }

    // ----------------------------------------------------------
    // The stores as org-io's files hold them.
    // ----------------------------------------------------------

    /// Put `store` in place of the held store with its Device key, or add it
    /// when none is held: org-io's reload of a Persona from its file after a
    /// failed write or removal. One store per Device key is kept (LLR-rdda86);
    /// nothing is sealed, no other store changes and no
    /// store is removed.
    pub fn restore_store(&mut self, store: PersonaStore) {
        match self.stores.iter().position(|held| held.device_key() == store.device_key()) {
            Some(index) => self.stores[index] = store,
            None => self.stores.push(store),
        }
    }

    /// Remove the store of `device_key` from the in-memory service, returning
    /// whether one was held. org-io calls it in two local-IO cases only: a
    /// refused keychain write for a new Persona, and a failed reload after a
    /// failed store write. It touches no file and no keychain item; this is
    /// the second exempt removal place of LLR-jd82t6.
    pub fn drop_store(&mut self, device_key: &DevicePublicKey) -> bool {
        let before = self.stores.len();
        self.stores.retain(|held| &held.device_key() != device_key);
        self.stores.len() != before
    }

    // ----------------------------------------------------------
    // Private helpers.
    // ----------------------------------------------------------

    /// The index of `persona_id`'s store.
    fn persona_index(&self, persona_id: &PersonaId) -> Result<usize, OrgNodeError> {
        self.stores
            .iter()
            .position(|store| &store.data().persona.persona_id == persona_id)
            .ok_or_else(|| OrgNodeError::Chain(format!("persona not found: {}", persona_id.as_str())))
    }

    fn find_persona(&self, persona_id: &PersonaId) -> Result<&PersonaRecord, OrgNodeError> {
        Ok(&self.stores[self.persona_index(persona_id)?].data().persona)
    }

    /// The index of the first store holding a record of `org_id`: the store
    /// of the Persona bound to it (`OrgNotOnChain` when none does).
    fn record_index(&self, org_id: OrgId) -> Result<usize, OrgNodeError> {
        self.stores
            .iter()
            .position(|store| store.data().orgs.iter().any(|record| record.org_id == org_id))
            .ok_or(OrgNodeError::OrgNotOnChain)
    }

    /// The index of the first store whose Persona is bound to `org_id`.
    fn bound_index(&self, org_id: OrgId) -> Option<usize> {
        self.stores.iter().position(|store| store.data().persona.org_id == Some(org_id))
    }

    fn find_org(&self, org_id: OrgId) -> Result<&OrgRecord, OrgNodeError> {
        find_record(self.stores[self.record_index(org_id)?].data(), org_id)
    }

    /// Refuse a Persona already bound to an Organisation (LLR-6z5xya,
    /// LLR-eyc4ud, REQ-yp75u9).
    fn ensure_unbound(&self, persona_id: &PersonaId) -> Result<(), OrgNodeError> {
        match self.find_persona(persona_id)?.org_id {
            Some(_) => Err(OrgNodeError::PersonaAlreadyBound { persona_id: persona_id.clone() }),
            None => Ok(()),
        }
    }

    /// Refuse `org_id` when a store of a Persona other than `persona_id`
    /// holds its record or expects its admission (LLR-yexf5m, REQ-uy7kfq).
    fn ensure_no_other_membership(&self, persona_id: &PersonaId, org_id: OrgId) -> Result<(), OrgNodeError> {
        let held_by_another = self.stores.iter().map(PersonaStore::data).any(|data| {
            &data.persona.persona_id != persona_id
                && (data.orgs.iter().any(|record| record.org_id == org_id)
                    || data.expected_admissions.iter().any(|expectation| expectation.org_id == org_id))
        });
        match held_by_another {
            true => Err(OrgNodeError::OrganisationHeldByAnotherPersona { org_id }),
            false => Ok(()),
        }
    }

    /// Record the expectation of `org_id` in the store at `index`, at most
    /// once (LLR-9zfnmb, LLR-7q9g9z).
    fn record_expectation<R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<StoreWrite, OrgNodeError> {
        let expectation = ExpectedAdmission { org_id };
        let ((), write) = self.change(index, rng, |data| {
            if !data.expected_admissions.contains(&expectation) {
                data.expected_admissions.push(expectation);
            }
            Ok(())
        })?;
        Ok(write)
    }

    /// The index of the store holding (`org_id`, `invite_id`) as
    /// outstanding, else `InviteNotOutstanding` (LLR-d55brn).
    fn outstanding_index(&self, org_id: OrgId, invite_id: InviteId) -> Result<usize, OrgNodeError> {
        self.stores
            .iter()
            .position(|store| store.data().outstanding_invites.contains(&(org_id, invite_id)))
            .ok_or(OrgNodeError::InviteNotOutstanding)
    }

    /// Make `change` to a copy of the data of the store at `index`, seal
    /// the copy, and only then adopt it: a refusal, of the change or of the
    /// seal, leaves the store as it was and returns no write (LLR-bvt5up,
    /// PR-jav8zn).
    fn change<T, R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        change: impl FnOnce(&mut StoreData) -> Result<T, OrgNodeError>,
    ) -> Result<(T, StoreWrite), OrgNodeError> {
        let mut data = self.stores[index].data().clone();
        let result = change(&mut data)?;
        let write = self.stores[index].seal_data(&data, rng)?;
        *self.stores[index].data_mut() = data;
        Ok((result, write))
    }

    /// Adopt a commit step's successor for the store at `index`: sealed and
    /// kept, or — when its Persona was forgotten with the Organisation —
    /// the store removed (LLR-pba7yu).
    fn adopt<R: RngCore + CryptoRng>(
        &mut self,
        index: usize,
        rng: &mut R,
        successor: Successor,
    ) -> Result<StoreChange, OrgNodeError> {
        match successor {
            Successor::Kept(data) | Successor::Removed(Some(data)) => {
                let write = self.stores[index].seal_data(&data, rng)?;
                *self.stores[index].data_mut() = data;
                Ok(StoreChange::Sealed(write))
            }
            Successor::Removed(None) => {
                let forgotten = self.stores.remove(index);
                Ok(StoreChange::Forgotten { device_key: forgotten.device_key() })
            }
        }
    }
}

/// `org_id`'s record in `data`, else `OrgNotOnChain`.
fn find_record(data: &StoreData, org_id: OrgId) -> Result<&OrgRecord, OrgNodeError> {
    data.orgs.iter().find(|record| record.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)
}

/// Bind the store's Persona to `org_id` as `member_id`, Active.
fn bind_persona(data: &mut StoreData, org_id: OrgId, member_id: MemberId) {
    data.persona.status = PersonaStatus::Active;
    data.persona.org_id = Some(org_id);
    data.persona.member_id = Some(member_id);
}

/// After a commit to `org_id` at `root`, drop every provisional update for
/// that Organisation built on another base (LLR-mkj4bz).
fn discard_orphans(data: &mut StoreData, org_id: OrgId, root: RootHash) {
    data.provisional_updates.retain(|provisional| provisional.org_id != Some(org_id) || provisional.base_root == Some(root));
}

/// Write a verified update to a held record — root, epoch, mark and
/// members together (LLR-cja9zv), and the Organisation key pair of the
/// update, replacing the one it held, no earlier one kept (LLR-6s785x,
/// LLR-ckk5nz, LLR-4kh9w9) — keep the verified Envelope's `change_set`,
/// replacing any earlier one (LLR-d9778a), and drop the provisional
/// updates it orphans (LLR-mkj4bz).
fn commit_held(
    data: &mut StoreData,
    org_id: OrgId,
    verified: &VerifiedUpdate,
    change_set: &[u8],
    org_private_key: OrgPrivateKey,
    org_pub_key: OrgPublicKey,
) -> Result<(), OrgNodeError> {
    let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
    let snapshots: Vec<MemberSnapshot> = verified.trie.members().iter().map(snapshot_of).collect();
    let rec = data.orgs.iter_mut().find(|record| record.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
    rec.root_hash = root;
    rec.epoch = verified.epoch;
    rec.last_seq = verified.seq_guard.last_seen();
    rec.trie_members = snapshots;
    rec.org_private_key = org_private_key;
    rec.org_pub_key = org_pub_key;
    rec.kept_change_set = Some(change_set.to_vec());
    discard_orphans(data, org_id, root);
    Ok(())
}

/// Commit received Organisation information that leaves this node listed
/// in its held record (LLR-jsx922); a revocation never reaches this, it
/// is decided by `revocation::accept` (LLR-pt32fx). The record takes
/// the key the message carried and the chain's public key, a pair
/// (REQ-ju6vn2, LLR-ckk5nz, LLR-4kh9w9), and keeps the received
/// Envelope's Change set (LLR-d9778a).
fn commit_received(
    data: &mut StoreData,
    org_id: OrgId,
    verified: &VerifiedUpdate,
    envelope: &Envelope,
    carried_key: OrgPrivateKey,
    chain_state: &OrgState,
) -> Result<(), OrgNodeError> {
    commit_held(data, org_id, verified, &envelope.delta_bytes, carried_key, chain_state.org_pub_key)
}

/// Refuse an Organisation private key whose X25519 public half is not the
/// Organisation public key the chain state read carries (LLR-ba2ejp,
/// RC-9cefcn).
fn check_carried_key(org_id: OrgId, key: &OrgPrivateKey, state: &OrgState) -> Result<(), OrgNodeError> {
    if key.x25519_keypair().org_public_key()? == state.org_pub_key {
        Ok(())
    } else {
        Err(OrgNodeError::OrgKeyMismatch { org_id })
    }
}

// ============================================================
// Outcomes.
// ============================================================

/// What a committed received message did (LLR-6qjmhf): a first admission
/// (also a founder's genesis commit), an update that keeps this node
/// listed, this node's own removal by an update, or a revocation notice
/// accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiveKind {
    Admitted,
    Updated,
    Removed,
    Revoked,
}

/// Outcome of `commit_checked` (LLR-6qjmhf) and of `commit_genesis`: the
/// Organisation, the epoch and root of the chain state it was decided
/// against, what it did, and the acknowledgements this node signed when it
/// was removed or revoked (else empty).
#[derive(Debug)]
pub struct ReceiveOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub kind: ReceiveKind,
    pub acknowledgements: Vec<Acknowledgement>,
}

/// One recipient's body of a committed update (LLR-gdfqn9): the Device it
/// goes to and the encoded Wire message. Not `Clone`: an
/// Organisation-information body carries the Organisation private key
/// (REQ-y7tsft); its `Debug` renders the recipient and the body's length
/// only.
pub struct OutgoingMessage {
    pub recipient: DevicePublicKey,
    pub body: Vec<u8>,
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

/// What `commit_update` (and a reconciled commit) returns (LLR-cmdrp9,
/// LLR-gdfqn9): the Organisation, the committed epoch and root, the sending
/// Persona, every recipient's body built inside the commit, and the
/// acknowledgements this node signed when the commit removed it
/// (LLR-b27jr6; else empty, and then there are no bodies). Not `Clone`.
#[derive(Debug)]
pub struct CommitOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub sender: PersonaId,
    pub outgoing: Vec<OutgoingMessage>,
    pub acknowledgements: Vec<Acknowledgement>,
}

// ============================================================
// What the check decided.
// ============================================================

/// What `check_received` decided (LLR-zetn7a): a message to decide against
/// the chain state of the Organisation it names, or an acknowledgement
/// decided against the store alone.
// One value per received message, moved once from the check to the commit:
// the checked message's size costs nothing worth a box.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Checked {
    /// Read the chain for `org_id()`, then call `commit_checked`.
    ReadChain(CheckedMessage),
    /// An acknowledgement `check_acknowledgement` accepted (LLR-3aysup,
    /// LLR-5azhry); no chain read, nothing written.
    Acknowledged(VerifiedAcknowledgement),
}

/// A message that passed `check_received`; opaque but for the Organisation
/// it names, and made only there (no public field, no other constructor).
///
/// It holds values taken from the store at check time (the record's trie
/// and marks). org-io's receive step releases the service between the check
/// and the commit for the one chain read, so a store change
/// may fall between them; the commit refuses the message when the record's
/// marks moved since the check (`RecordMovedSinceCheck`, PR-erskh4), and
/// otherwise judges these values against the chain state read after the
/// check.
pub struct CheckedMessage {
    kind: CheckedKind,
    record_at_check: Option<RecordMarks>,
}

/// The marks of a held Organisation record that a check reads and a commit
/// changes: its epoch, root and sequence guard (PR-erskh4).
#[derive(Clone, Copy, PartialEq, Eq)]
struct RecordMarks {
    epoch: Epoch,
    root: RootHash,
    last_seq: SequenceNumber,
}

// One checked message per received message, moved once to the commit.
#[allow(clippy::large_enum_variant)]
enum CheckedKind {
    /// `check_notice` and the sender rule passed.
    Revocation { notice: RevocationNotice },
    /// The sender rule (held record) or the expectation gate (first
    /// admission) passed, the snapshot decoded, `check_chain_free` passed.
    /// `expectation` is the one a first admission matched, `None` for a held
    /// record.
    OrgInformation {
        envelope: Envelope,
        carried_key: OrgPrivateKey,
        local_trie: Trie,
        ctx: VerifyContext,
        expectation: Option<ExpectedAdmission>,
    },
}

impl CheckedKind {
    /// The Organisation the checked message names.
    fn org_id(&self) -> OrgId {
        match self {
            CheckedKind::Revocation { notice } => notice.org_id,
            CheckedKind::OrgInformation { envelope, .. } => envelope.org_id,
        }
    }
}

impl CheckedMessage {
    /// The Organisation whose chain state `commit_checked` needs.
    pub fn org_id(&self) -> OrgId {
        self.kind.org_id()
    }
}

/// Names the Organisation and the message kind only: the checked message
/// holds the carried Organisation private key and the record's trie,
/// neither printed.
impl std::fmt::Debug for CheckedMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.kind {
            CheckedKind::Revocation { .. } => "Revocation",
            CheckedKind::OrgInformation { .. } => "OrgInformation",
        };
        formatter.debug_struct("CheckedMessage").field("org_id", &self.org_id()).field("kind", &kind).finish()
    }
}

// ============================================================
// The commit step and the removal step, over store values.
// ============================================================

/// What a commit leaves of the store it was decided over: the committed
/// successor (this node still listed), or the removal step's —
/// `forget_organisation`'s result, `None` when the store's Persona went with
/// the Organisation.
pub(crate) enum Successor {
    Kept(StoreData),
    Removed(Option<StoreData>),
}

/// What `commit_step` produced: the successor store and the outcome; a
/// removal's outcome holds the acknowledgements it signed.
pub(crate) struct Committed {
    pub(crate) successor: Successor,
    pub(crate) outcome: CommitOutcome,
}

/// Commit `update`, this node's own provisional update for `org_id`,
/// against `chain`, the state read for it: the Envelope built from the
/// update, verified against `store`'s record and `chain` (LLR-cmdrp9,
/// LLR-6s785x); then the record takes the update's key pair (REQ-jy6ybw)
/// and every recipient's body is built with that key, the notices of
/// exactly this update among them (`outgoing_bodies`; LLR-gdfqn9,
/// LLR-a8z7r5), or, when the store's Persona bound to `org_id` is no longer
/// listed, the one removal step with the secrets `device_secrets` yields for
/// the Device keys it names, called on that path alone so a commit that
/// keeps this node listed never obtains a device secret (LLR-b27jr6, LLR-jd82t6, LLR-uw7nmv). `commit_update`
/// and `reconcile` both run it (LLR-fm38ww). A refusal returns the error
/// alone; `store` is borrowed and unchanged.
pub(crate) fn commit_step(
    store: &StoreData,
    org_id: OrgId,
    update: &ProvisionalUpdate,
    chain: &OrgState,
    device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
) -> Result<Committed, OrgNodeError> {
    let rec = find_record(store, org_id)?;
    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change.clone() else {
        return Err(OrgNodeError::NoProvisionalUpdate);
    };
    let envelope = Envelope { org_id, parent_seq: update.seq, delta_bytes: change_set };
    let ctx = VerifyContext {
        expected_org_id: org_id,
        seq_guard: SeqGuard::from_last_seen(rec.last_seq),
        last_committed_epoch: rec.epoch,
    };
    let local = trie_from_snapshots(&rec.trie_members)?;
    let verified = verify_envelope_against_chain(&local, &envelope, &ctx, Some(*chain))?;
    let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
    let (successor, outgoing, acknowledgements) = if still_member(store, org_id, &verified.trie) {
        let outgoing = outgoing_bodies(store, rec, &verified.trie, envelope.clone(), &org_private_key)?;
        let mut successor = store.clone();
        commit_held(&mut successor, org_id, &verified, &envelope.delta_bytes, org_private_key, update.org_pub_key)?;
        (Successor::Kept(successor), outgoing, Vec::new())
    } else {
        // This node was removed: it forgets the Organisation and sends
        // nothing (LLR-b27jr6).
        let (successor, acknowledgements) = removal_step(store, org_id, chain, device_secrets)?;
        (Successor::Removed(successor), Vec::new(), acknowledgements)
    };
    let outcome = CommitOutcome {
        org_id,
        epoch: verified.epoch,
        root,
        sender: store.persona.persona_id.clone(),
        outgoing,
        acknowledgements,
    };
    Ok(Committed { successor, outcome })
}

/// Every recipient's body of a commit that keeps this node listed, built
/// from the record as committed (`committed_trie`) and with `org_private_key`,
/// the key the commit stores (LLR-gdfqn9, LLR-6ymd6d; PR-gnh3j2): one
/// Organisation-information body — `envelope`, `previous` as it stood
/// before the commit (LLR-bg3vsw) and that key (REQ-szq3ud) — for each
/// Device the committed record lists other than the sending Persona's own,
/// in record order; then one revocation body, the notice of exactly this
/// update (LLR-a8z7r5, LLR-kr5t6f), for each Device `previous` listed and
/// the committed record does not (REQ-3dsweu, LLR-8hdu9x). No body carries
/// an invite identifier (LLR-48jakr). Each is `encode_message`'s output.
fn outgoing_bodies(
    store: &StoreData,
    previous: &OrgRecord,
    committed_trie: &Trie,
    envelope: Envelope,
    org_private_key: &OrgPrivateKey,
) -> Result<Vec<OutgoingMessage>, OrgNodeError> {
    let committed =
        OrgRecord { trie_members: committed_trie.members().iter().map(snapshot_of).collect(), ..previous.clone() };
    let information = encode_message(&WireMessage::OrgInformation {
        envelope,
        record_snapshot: encode_record_snapshot(&previous.trie_members)?,
        org_private_key: org_private_key.clone(),
    })?;
    let own_device = store.persona.device_key;
    let mut outgoing: Vec<OutgoingMessage> = committed
        .trie_members
        .iter()
        .flat_map(|member| member.device_keys.iter().copied())
        .filter(|device| *device != own_device)
        .map(|recipient| OutgoingMessage { recipient, body: information.clone() })
        .collect();
    for notice in crate::revocation::notices_for(previous, &committed, committed_trie)? {
        let recipient = notice.device;
        outgoing.push(OutgoingMessage { recipient, body: encode_message(&WireMessage::Revocation(notice))? });
    }
    Ok(outgoing)
}

/// The one removal step of every commit path (LLR-jd82t6): a commit removed
/// this node from `org_id`, so it signs one acknowledgement per Persona
/// bound to it against `chain`, the state the commit was decided against,
/// with the secrets `device_secrets` yields for those Personas' Device keys,
/// then returns `forget_organisation(org_id)` with the acknowledgements
/// (LLR-b27jr6, LLR-6p4pj2, LLR-hby4jr). A signing refusal returns the error
/// alone.
fn removal_step(
    store: &StoreData,
    org_id: OrgId,
    chain: &OrgState,
    device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
) -> Result<(Option<StoreData>, Vec<Acknowledgement>), OrgNodeError> {
    let secrets = device_secrets(&crate::revocation::bound_devices(store, org_id));
    let acknowledgements = crate::revocation::sign_acknowledgements(store, org_id, chain, secrets)?;
    Ok((store.forget_organisation(org_id), acknowledgements))
}

/// The member-sender rule (owner ruling R1 of 2026-10-07): `sender` must be
/// in some member snapshot's `device_keys` of `record`, else
/// `SenderNotListed`. Called for Organisation information about a held
/// Organisation (LLR-2r2fha) and for a revocation (LLR-kzgjz8), never for a
/// first admission or an acknowledgement.
fn sender_listed(record: &OrgRecord, org_id: OrgId, sender: &DevicePublicKey) -> Result<(), OrgNodeError> {
    if record.trie_members.iter().any(|member| member.device_keys.contains(sender)) {
        Ok(())
    } else {
        Err(OrgNodeError::SenderNotListed { org_id })
    }
}

/// Whether the store's Persona is bound to `org_id` and has its device in
/// `trie`.
fn still_member(store: &StoreData, org_id: OrgId, trie: &Trie) -> bool {
    let persona = &store.persona;
    persona.org_id == Some(org_id) && trie.members().iter().any(|member| member.has_p2p_device(&persona.device_key))
}

// ============================================================
// Utility helpers.
// ============================================================

/// A Persona identifier from its Member key: the first 16 bytes, in hex.
fn persona_id_for(member_key: &PersonPublicKey) -> PersonaId {
    use std::fmt::Write as _;
    PersonaId::new(member_key.as_bytes().iter().take(16).fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    }))
}

/// A fresh `MemberId`: 32 bytes from the caller's cryptographic random
/// source, never derived from a key (REQ-d9g6nt).
fn fresh_member_id<R: RngCore + CryptoRng>(rng: &mut R) -> MemberId {
    let mut id = [0u8; 32];
    rng.fill_bytes(&mut id);
    MemberId::new(id)
}
