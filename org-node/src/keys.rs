//! ed25519 keypairs for members and devices. A device's verifying key is
//! both its P2pDeviceKey (in the trie) and — in a later phase — its iroh
//! NodeId. The member's verifying key is the P2pMemberKey used to sign deltas.
//! A seed becomes a key pair only through `MemberSeed` or `DeviceSeed`
//! (LLR-56hc77).
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use org_members::{P2pDeviceKey, P2pMemberKey};

use crate::types::{DeviceSeed, MemberSeed};

/// An ed25519 keypair held locally. Wraps a dalek SigningKey.
#[derive(Clone, Debug)]
pub struct SigningKeypair(SigningKey);

impl MemberSeed {
    /// The Member's signing key pair (LLR-56hc77).
    pub fn signing_keypair(&self) -> SigningKeypair {
        SigningKeypair(SigningKey::from_bytes(self.expose_secret()))
    }
}

impl DeviceSeed {
    /// The device's signing key pair (LLR-56hc77).
    pub fn signing_keypair(&self) -> SigningKeypair {
        SigningKeypair(SigningKey::from_bytes(self.expose_secret()))
    }
}

impl SigningKeypair {
    /// Generate from a CSPRNG. (Tests use rand; production wires this to the OS RNG.)
    pub fn generate<R: rand_core::CryptoRng + rand_core::RngCore>(rng: &mut R) -> Self {
        Self(SigningKey::generate(rng))
    }

    /// This key pair's seed, held as a Member seed (LLR-56hc77).
    pub fn member_seed(&self) -> MemberSeed {
        MemberSeed::from(self.0.to_bytes())
    }

    /// This key pair's seed, held as a device seed (LLR-56hc77).
    pub fn device_seed(&self) -> DeviceSeed {
        DeviceSeed::from(self.0.to_bytes())
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.0.verifying_key()
    }

    /// As a member-as-a-group key for the trie.
    pub fn member_key(&self) -> P2pMemberKey {
        P2pMemberKey::new(self.verifying_key())
    }

    /// As a device key for the trie / iroh identity.
    pub fn device_key(&self) -> P2pDeviceKey {
        P2pDeviceKey::new(self.verifying_key())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.0.sign(msg)
    }
}

/// Verify a signature against an already-known verifying key.
pub fn verify(vk: &VerifyingKey, msg: &[u8], sig: &Signature) -> bool {
    vk.verify(msg, sig).is_ok()
}

