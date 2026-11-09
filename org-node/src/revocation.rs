//! Revocation notices (SDD-gatf4h): the revocation a revoked Device receives
//! and the decision it acts on; and the removal acknowledgement's signature
//! (SDD-8zzmav). Pure functions over values (LLR-g2jeqd).
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

/// The signing domain of an acknowledgement (LLR-bs6yxf).
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
/// (REQ-h29c8e). Not evidence of erasure (RC-eydn8t).
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
    /// of `(org_id, member_id, device, epoch, root)`. LLR-bs6yxf.
    ///
    /// Written by hand so that no step can fail (PR-y6ay44): postcard
    /// encodes each fixed-size byte array as its bytes, with no length, and
    /// the epoch as its LEB128 varint.
    pub fn signed_bytes(&self) -> Vec<u8> {
        let mut bytes = ACK_DOMAIN.to_vec();
        bytes.extend_from_slice(self.org_id.as_bytes());
        bytes.extend_from_slice(self.member_id.as_bytes());
        bytes.extend_from_slice(self.device.as_bytes());
        push_varint(&mut bytes, self.epoch.get());
        bytes.extend_from_slice(self.root.as_bytes());
        bytes
    }
}

/// Appends the epoch number `epoch` as postcard's unsigned varint: LEB128,
/// seven bits a byte, low group first, the high bit set on every byte but
/// the last.
fn push_varint(bytes: &mut Vec<u8>, mut epoch: u64) {
    while epoch >= 0x80 {
        bytes.push((epoch & 0x7f) as u8 | 0x80);
        epoch >>= 7;
    }
    bytes.push(epoch as u8);
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

/// The Persona a notice is for, decided with no Organisation state: the
/// store must hold the Organisation, and a Persona bound to it must be the
/// notice's Member on the notice's Device. LLR-r7zm39.
#[cfg(feature = "app")]
pub fn check_notice<'a>(store: &'a StoreData, notice: &RevocationNotice) -> Result<&'a PersonaRecord, OrgNodeError> {
    let org_id = notice.org_id;
    if store.record.as_ref().is_none_or(|record| record.org_id != org_id) {
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

/// Decides a kept notice against the chain state the caller read (LLR-rhsfw9's refusals, in order;
/// LLR-jhy6t6): `check_notice`; no state (`OrgNotOnChain`); a state older than the record
/// (`StaleChainState`); at the record's epoch with another root (`ChainStateConflict`); a proof that
/// does not hold under the chain's root (`RevocationProofRefused`). Asks for no secret and changes nothing.
#[cfg(feature = "app")]
pub fn accept(store: &StoreData, notice: &RevocationNotice, chain: Option<OrgState>) -> Result<(), OrgNodeError> {
    check_notice(store, notice)?;
    let org_id = notice.org_id;
    let chain = chain.ok_or(OrgNodeError::OrgNotOnChain)?;
    let record = store.record.as_ref().ok_or(OrgNodeError::RevocationNotHeld { org_id })?;
    if chain.epoch < record.epoch {
        return Err(OrgNodeError::StaleChainState { org_id, chain_epoch: chain.epoch, record_epoch: record.epoch });
    }
    // At the record's own epoch the chain must hold the record's root, as
    // `verify` requires (owner ruling R3: false deletion extremely unlikely).
    if chain.epoch == record.epoch && chain.root_hash != record.root_hash {
        return Err(OrgNodeError::ChainStateConflict { org_id });
    }
    notice
        .proof
        .verify::<Blake3Hasher>(&chain.root_hash, &notice.member_id, &notice.device)
        .map_err(|cause| OrgNodeError::RevocationProofRefused { org_id, cause })
}

/// This Persona's one acknowledgement of its own removal from `org_id`, naming `chain`'s epoch and root,
/// signed with `secret` (LLR-5tvn7g, LLR-bs6yxf; Ed25519, deterministic). `secret` must be the Persona's
/// (`DeviceSecretNotSupplied`).
#[cfg(feature = "app")]
pub fn sign_acknowledgement(
    persona: &PersonaRecord,
    org_id: OrgId,
    chain: &OrgState,
    secret: &DeviceSecret,
) -> Result<Acknowledgement, OrgNodeError> {
    let member_id = persona.member_id.ok_or(OrgNodeError::OrgNotHeld { org_id })?;
    if secret.device_key() != persona.device_key {
        return Err(OrgNodeError::DeviceSecretNotSupplied { org_id });
    }
    let mut acknowledgement = Acknowledgement {
        org_id,
        member_id,
        device: persona.device_key,
        epoch: chain.epoch,
        root: chain.root_hash,
        signature: Signature64([0; 64]),
    };
    acknowledgement.signature = Signature64(secret.signing_keypair().sign(&acknowledgement.signed_bytes()).to_bytes());
    Ok(acknowledgement)
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
        .record
        .as_ref()
        .filter(|record| record.org_id == org_id)
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
