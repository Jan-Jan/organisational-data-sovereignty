//! SDD-uck4tz: the revocation a revoked Device receives, the decision it acts
//! on, and its acknowledgement. Pure functions over values (LLR-xgefn8).
use org_members::{AbsenceProof, DevicePublicKey, MemberId, RootHash};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "app")]
use crate::error::OrgNodeError;
use crate::ids::OrgId;
#[cfg(feature = "app")]
use crate::chain::OrgState;
#[cfg(feature = "app")]
use crate::store::{OrgRecord, PersonaRecord, StoreData};
use crate::types::Epoch;
#[cfg(feature = "app")]
use crate::types::DeviceSeed;
#[cfg(feature = "app")]
use crate::verify::Trie;
#[cfg(feature = "app")]
use org_members::hasher::Blake3Hasher;

/// The signing domain of an acknowledgement (LLR-gbe9bt).
pub const ACK_DOMAIN: &[u8] = b"ods/org-node/revocation-acknowledgement/v1";

/// What a revoked Device receives (REQ-ps2gy2): its identity and an absence
/// proof from the committed record, nothing else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationNotice {
    pub org_id: OrgId,
    pub member_id: MemberId,
    pub device: DevicePublicKey,
    pub proof: AbsenceProof,
}

/// The 64 bytes of an ed25519 signature. Its serde form is a byte sequence;
/// decoding refuses any length but 64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signature64(pub [u8; 64]);

impl Serialize for Signature64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Signature64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        let length = bytes.len();
        let signature: [u8; 64] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::invalid_length(length, &"64 signature bytes"))?;
        Ok(Self(signature))
    }
}

/// A revoked Device's signed statement that it acted on its revocation
/// (REQ-y99c9w). Not evidence of erasure (RC-eydn8t).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub org_id: OrgId,
    pub member_id: MemberId,
    pub device: DevicePublicKey,
    pub epoch: Epoch,
    pub root: RootHash,
    pub signature: Signature64,
}

impl Acknowledgement {
    /// The bytes the Device signs: [`ACK_DOMAIN`], then the postcard encoding
    /// of `(org_id, member_id, device, epoch, root)`. LLR-gbe9bt.
    pub fn signed_bytes(&self) -> Vec<u8> {
        let mut bytes = ACK_DOMAIN.to_vec();
        if let Ok(tuple) = postcard::to_allocvec(&(self.org_id, self.member_id, self.device, self.epoch, self.root)) {
            bytes.extend(tuple);
        }
        bytes
    }
}

/// One notice per Device listed in `previous` and in no Member of
/// `committed`, in Member then slot order, each with an absence proof from
/// `committed_trie`. LLR-kr5t6f. Needs the store, so `app` only; the
/// notice and the acknowledgement are `transport`'s, which the Wire message
/// carries.
#[cfg(feature = "app")]
pub fn notices_for(
    previous: &OrgRecord,
    committed: &OrgRecord,
    committed_trie: &Trie,
) -> Result<Vec<RevocationNotice>, OrgNodeError> {
    let still_listed =
        |device: &DevicePublicKey| committed.trie_members.iter().any(|member| member.device_keys.contains(device));
    let mut notices = Vec::new();
    for member in &previous.trie_members {
        for device in member.device_keys.iter().filter(|device| !still_listed(device)) {
            let proof = committed_trie.prove_absent(&member.id, device).map_err(OrgNodeError::Trie)?;
            notices.push(RevocationNotice { org_id: committed.org_id, member_id: member.id, device: *device, proof });
        }
    }
    Ok(notices)
}

/// A notice `accept` took: the store without the Organisation and the
/// acknowledgements signed before it was forgotten (LLR-r8qhky).
#[cfg(feature = "app")]
pub struct Accepted {
    pub store: StoreData,
    pub acknowledgements: Vec<Acknowledgement>,
}

/// The Persona a notice is for, decided with no Organisation state: the
/// store must hold the Organisation, and a Persona bound to it must be the
/// notice's Member on the notice's Device. LLR-r7zm39.
#[cfg(feature = "app")]
pub fn check_notice<'a>(store: &'a StoreData, notice: &RevocationNotice) -> Result<&'a PersonaRecord, OrgNodeError> {
    let org_id = notice.org_id;
    if !store.orgs.iter().any(|record| record.org_id == org_id) {
        return Err(OrgNodeError::RevocationNotHeld { org_id });
    }
    store
        .personas
        .iter()
        .find(|persona| {
            persona.org_id == Some(org_id)
                && persona.member_id == Some(notice.member_id)
                && persona_device(persona) == Some(notice.device)
        })
        .ok_or(OrgNodeError::RevocationNotForThisDevice { org_id })
}

/// Decides a notice against the Organisation state the caller read: the
/// state must exist, be no older than the record, hold the record's root
/// when at the record's epoch, and the notice's proof
/// must show the Device absent under its root. On success the seed source
/// is called, the acknowledgements are signed, then the Organisation is
/// forgotten; on any refusal the error alone is returned and the seed
/// source is never called. LLR-tx8ruv, LLR-r8qhky,
/// LLR-uw7nmv.
#[cfg(feature = "app")]
pub fn accept(
    store: &StoreData,
    notice: &RevocationNotice,
    chain: Option<OrgState>,
    device_seeds: impl FnOnce() -> Vec<DeviceSeed>,
) -> Result<Accepted, OrgNodeError> {
    check_notice(store, notice)?;
    let org_id = notice.org_id;
    let chain = chain.ok_or(OrgNodeError::OrgNotOnChain)?;
    let record = store
        .orgs
        .iter()
        .find(|record| record.org_id == org_id)
        .ok_or(OrgNodeError::RevocationNotHeld { org_id })?;
    if chain.epoch < record.epoch {
        return Err(OrgNodeError::StaleChainState { org_id, chain_epoch: chain.epoch, record_epoch: record.epoch });
    }
    // At the record's own epoch the chain must hold the record's root, as
    // `reconcile` requires (owner ruling R3: false deletion extremely unlikely).
    if chain.epoch == record.epoch && chain.root_hash != record.root_hash {
        return Err(OrgNodeError::ChainStateConflict { org_id });
    }
    notice
        .proof
        .verify::<Blake3Hasher>(&chain.root_hash, &notice.member_id, &notice.device)
        .map_err(|cause| OrgNodeError::RevocationProofRefused { org_id, cause })?;
    // Every refusal check has passed: only now are the device seeds read.
    let acknowledgements = sign_acknowledgements(store, org_id, &chain, device_seeds())?;
    Ok(Accepted { store: store.forget_organisation(org_id), acknowledgements })
}

/// One acknowledgement per Persona bound to `org_id`, in store order (every
/// binding — `commit_genesis` and a first admission — sets the MemberId with
/// the Organisation, so the `member_id` filter below skips no bound Persona;
/// PR-eqs4fs), naming `verified`'s epoch and root and signed with the seed
/// in `device_seeds` whose public key is that Persona's Device. Refuses
/// with `DeviceSecretNotSupplied` when such a Persona has no seed there.
/// The seeds are moved in and dropped on return. LLR-hby4jr.
#[cfg(feature = "app")]
pub fn sign_acknowledgements(
    store: &StoreData,
    org_id: OrgId,
    verified: &OrgState,
    device_seeds: Vec<DeviceSeed>,
) -> Result<Vec<Acknowledgement>, OrgNodeError> {
    let mut acknowledgements = Vec::new();
    for persona in store.personas.iter().filter(|persona| persona.org_id == Some(org_id)) {
        let Some(member_id) = persona.member_id else { continue };
        let device = persona_device(persona).ok_or(OrgNodeError::DeviceSecretNotSupplied { org_id })?;
        let keypair = device_seeds
            .iter()
            .map(DeviceSeed::signing_keypair)
            .find(|keypair| keypair.device_key().ok() == Some(device))
            .ok_or(OrgNodeError::DeviceSecretNotSupplied { org_id })?;
        let mut acknowledgement = Acknowledgement {
            org_id,
            member_id,
            device,
            epoch: verified.epoch,
            root: verified.root_hash,
            signature: Signature64([0; 64]),
        };
        acknowledgement.signature = Signature64(keypair.sign(&acknowledgement.signed_bytes()).to_bytes());
        acknowledgements.push(acknowledgement);
    }
    Ok(acknowledgements)
}

/// An acknowledgement `check_acknowledgement` accepted. Only that function
/// constructs one (LLR-5azhry).
#[cfg(feature = "app")]
#[derive(Debug)]
pub struct VerifiedAcknowledgement(Acknowledgement);

#[cfg(feature = "app")]
impl VerifiedAcknowledgement {
    pub fn into_inner(self) -> Acknowledgement {
        self.0
    }
}

/// Checks a received acknowledgement against the store alone, cheapest
/// first: the store holds the Organisation's record; `sender`, the Device
/// the transport authenticated for the delivering connection, is the Device
/// the acknowledgement names; the acknowledgement's epoch is not above that
/// record's; no Member of the record lists its Device; its signature
/// verifies strictly under that Device. The store is not changed.
/// LLR-5azhry.
#[cfg(feature = "app")]
pub fn check_acknowledgement(
    store: &StoreData,
    sender: DevicePublicKey,
    ack: Acknowledgement,
) -> Result<VerifiedAcknowledgement, OrgNodeError> {
    let org_id = ack.org_id;
    let record = store
        .orgs
        .iter()
        .find(|record| record.org_id == org_id)
        .ok_or(OrgNodeError::AcknowledgementNotHeld { org_id })?;
    if sender != ack.device {
        return Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id });
    }
    if ack.epoch > record.epoch {
        return Err(OrgNodeError::AcknowledgementFromFuture { org_id });
    }
    if record.trie_members.iter().any(|member| member.device_keys.contains(&ack.device)) {
        return Err(OrgNodeError::AcknowledgementForListedDevice { org_id });
    }
    let signature = ed25519_dalek::Signature::from_bytes(&ack.signature.0);
    ack.device
        .verifying_key()
        .verify_strict(&ack.signed_bytes(), &signature)
        .map_err(|_| OrgNodeError::AcknowledgementSignatureInvalid { org_id })?;
    Ok(VerifiedAcknowledgement(ack))
}

/// A Persona's DevicePublicKey. Today derived from its stored device seed;
/// stage S4 replaces this body with the stored public key. The one place a
/// Persona's Device is derived, for the receive and commit paths too.
#[cfg(feature = "app")]
pub(crate) fn persona_device(persona: &PersonaRecord) -> Option<DevicePublicKey> {
    persona.device_seed.signing_keypair().device_key().ok()
}
