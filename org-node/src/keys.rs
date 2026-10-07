//! Key material held locally. A device's key is ed25519: its verifying key
//! is its DevicePublicKey in the trie and its iroh EndpointId. A
//! Member-as-a-group key and the Organisation public key (REQ-ech45n) are
//! X25519 key-agreement keys, each derived from a 32-byte secret (a member
//! seed, the Organisation private key) by RFC 7748 clamping and
//! multiplication by the base point. A seed becomes a key pair only through
//! its secret type, and a key pair hands its seed back only as one
//! (LLR-56hc77).
use core::fmt;

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use org_members::{DevicePublicKey, OrgMembersError, PersonPublicKey};
use rand_core::{CryptoRng, RngCore};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::OrgNodeError;
use crate::types::{DeviceSeed, MemberSeed, OrgPrivateKey, OrgPublicKey};

/// An ed25519 keypair held locally: a device's identity. Wraps a dalek SigningKey.
#[derive(Clone, Debug)]
pub struct SigningKeypair(SigningKey);

impl MemberSeed {
    /// The Member's X25519 key pair: its Member-as-a-group key (LLR-56hc77,
    /// LLR-98ufry).
    pub fn x25519_keypair(&self) -> X25519Keypair {
        X25519Keypair(*self.expose_secret())
    }
}

impl OrgPrivateKey {
    /// The Organisation's X25519 key pair (REQ-ech45n, LLR-98ufry).
    pub fn x25519_keypair(&self) -> X25519Keypair {
        X25519Keypair(*self.expose_secret())
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
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self(SigningKey::generate(rng))
    }

    /// This key pair's seed, held as a device seed (LLR-56hc77).
    pub fn device_seed(&self) -> DeviceSeed {
        DeviceSeed::from(self.0.to_bytes())
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.0.verifying_key()
    }

    /// The ed25519 signature of `message` under this key pair.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.0.sign(message)
    }

    /// As a DevicePublicKey for the trie / iroh identity, through `person`'s parse.
    pub fn device_key(&self) -> Result<DevicePublicKey, OrgMembersError> {
        Ok(DevicePublicKey::try_from(self.verifying_key())?)
    }
}

/// An X25519 secret held locally, as the 32 bytes it is persisted as. Not
/// `Clone`, and its bytes are overwritten with zeros when it is dropped
/// (LLR-98ufry).
pub struct X25519Keypair([u8; 32]);

impl X25519Keypair {
    /// 32 bytes from the caller's cryptographic random source.
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        Self(seed)
    }

    /// This key pair's secret, held as a Member seed for at-rest persistence
    /// (LLR-56hc77).
    pub fn member_seed(&self) -> MemberSeed {
        MemberSeed::from(self.0)
    }

    /// This key pair's secret, held as an Organisation private key for
    /// at-rest persistence (LLR-56hc77).
    pub fn org_private_key(&self) -> OrgPrivateKey {
        OrgPrivateKey::from(self.0)
    }

    /// The X25519 public key: the clamped secret times the base point
    /// (RFC 7748 §5).
    pub fn public_bytes(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(self.0).to_bytes()
    }

    /// As a Member-as-a-group key for the trie, through `person`'s parse.
    pub fn member_key(&self) -> Result<PersonPublicKey, OrgMembersError> {
        Ok(PersonPublicKey::parse(&self.public_bytes())?)
    }

    /// As an Organisation public key, through `OrgPublicKey::parse`.
    pub fn org_public_key(&self) -> Result<OrgPublicKey, OrgNodeError> {
        OrgPublicKey::parse(&self.public_bytes())
    }
}

/// Redacted: the secret never reaches a log.
impl fmt::Debug for X25519Keypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("X25519Keypair(..)")
    }
}

impl Zeroize for X25519Keypair {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for X25519Keypair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl ZeroizeOnDrop for X25519Keypair {}
