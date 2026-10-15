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
use crate::keys::DeviceSecret;
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
/// acknowledgements signed before it was forgotten (LLR-r8qhky). The store
/// is `forget_organisation`'s result, `None` when the store's
/// one Persona went with the Organisation (the caller then removes the
/// store, `StoreChange::Forgotten`).
#[cfg(feature = "app")]
pub struct Accepted {
    pub store: Option<StoreData>,
    pub acknowledgements: Vec<Acknowledgement>,
}

/// The Device keys of the Personas of `store` bound to `org_id`: what a
/// signing path asks its caller's `device_secrets` closure for (LLR-uw7nmv,
/// LLR-hby4jr).
#[cfg(feature = "app")]
pub(crate) fn bound_devices(store: &StoreData, org_id: OrgId) -> Vec<DevicePublicKey> {
    if store.persona.org_id == Some(org_id) {
        vec![store.persona.device_key]
    } else {
        Vec::new()
    }
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
    let persona = &store.persona;
    if persona.org_id == Some(org_id) && persona.member_id == Some(notice.member_id) && persona.device_key == notice.device
    {
        Ok(persona)
    } else {
        Err(OrgNodeError::RevocationNotForThisDevice { org_id })
    }
}

/// The index, among `stores`, of the store whose Persona a notice is for:
/// the first that `check_notice` accepts; else the last
/// `RevocationNotForThisDevice` refusal, or `RevocationNotHeld` when no
/// store holds the Organisation. LLR-r7zm39, LLR-38e2kn. It is here, not in
/// the service, so that the unheld refusal is raised in this file only
/// (PR-q8r32t).
#[cfg(feature = "app")]
pub(crate) fn notice_store<'a>(
    stores: impl Iterator<Item = &'a StoreData>,
    notice: &RevocationNotice,
) -> Result<usize, OrgNodeError> {
    let mut refusal = OrgNodeError::RevocationNotHeld { org_id: notice.org_id };
    for (index, store) in stores.enumerate() {
        match check_notice(store, notice) {
            Ok(_) => return Ok(index),
            Err(error @ OrgNodeError::RevocationNotForThisDevice { .. }) => refusal = error,
            Err(_) => {}
        }
    }
    Err(refusal)
}

/// Decides a notice against the Organisation state the caller read: the
/// state must exist, be no older than the record, hold the record's root
/// when at the record's epoch, and the notice's proof
/// must show the Device absent under its root. On success the secret source
/// is called, with the Device keys of the bound Personas, the
/// acknowledgements are signed, then the Organisation is forgotten; on any
/// refusal the error alone is returned and the secret source is never
/// called. LLR-tx8ruv, LLR-r8qhky, LLR-uw7nmv.
#[cfg(feature = "app")]
pub fn accept(
    store: &StoreData,
    notice: &RevocationNotice,
    chain: Option<OrgState>,
    device_secrets: impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>,
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
    // Every refusal check has passed: only now are the device secrets read.
    let acknowledgements = sign_acknowledgements(store, org_id, &chain, device_secrets(&bound_devices(store, org_id)))?;
    Ok(Accepted { store: store.forget_organisation(org_id), acknowledgements })
}

/// One acknowledgement per Persona bound to `org_id`, in store order (every
/// binding — `commit_genesis` and a first admission — sets the MemberId with
/// the Organisation, so the `member_id` filter below skips no bound Persona;
/// PR-eqs4fs), naming `verified`'s epoch and root and signed with the
/// secret in `device_secrets` whose Device key is that Persona's. Refuses
/// with `DeviceSecretNotSupplied` when such a Persona has no secret there.
/// The secrets are moved in and dropped, zeroised, on return. LLR-hby4jr.
/// A store holds one Persona, so at most one acknowledgement.
#[cfg(feature = "app")]
pub fn sign_acknowledgements(
    store: &StoreData,
    org_id: OrgId,
    verified: &OrgState,
    device_secrets: Vec<DeviceSecret>,
) -> Result<Vec<Acknowledgement>, OrgNodeError> {
    let persona = &store.persona;
    if persona.org_id != Some(org_id) {
        return Ok(Vec::new());
    }
    let Some(member_id) = persona.member_id else { return Ok(Vec::new()) };
    let device = persona.device_key;
    let keypair = device_secrets
        .iter()
        .find(|secret| secret.device_key() == device)
        .map(DeviceSecret::signing_keypair)
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
    Ok(vec![acknowledgement])
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
