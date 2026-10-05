//! ed25519 keypairs for members and devices. A device's verifying key is
//! both its P2pDeviceKey (in the trie) and — in a later phase — its iroh
//! NodeId. The member's verifying key is the P2pMemberKey used to sign deltas.
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use org_members::{P2pDeviceKey, P2pMemberKey};

/// An ed25519 keypair held locally. Wraps a dalek SigningKey.
#[derive(Clone, Debug)]
pub struct SigningKeypair(SigningKey);

impl SigningKeypair {
    /// Generate from a CSPRNG. (Tests use rand; production wires this to the OS RNG.)
    pub fn generate<R: rand_core::CryptoRng + rand_core::RngCore>(rng: &mut R) -> Self {
        Self(SigningKey::generate(rng))
    }

    /// Reconstruct from the 32-byte secret seed (for persisted keys).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(SigningKey::from_bytes(&seed))
    }

    /// The 32-byte secret seed, for at-rest persistence. Handle as a secret.
    pub fn to_seed(&self) -> [u8; 32] {
        self.0.to_bytes()
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

