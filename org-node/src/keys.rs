//! Key material held locally. A device's key is ed25519: its verifying key
//! is its DevicePublicKey in the trie and its endpoint identity (org-io). A
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
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::error::OrgNodeError;
use crate::types::{MemberSeed, OrgPrivateKey, OrgPublicKey};

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

impl SigningKeypair {
    /// Generate from a CSPRNG. (Tests use rand; production wires this to the OS RNG.)
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self(SigningKey::generate(rng))
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.0.verifying_key()
    }

    /// The ed25519 signature of `message` under this key pair.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.0.sign(message)
    }

    /// As a DevicePublicKey for the trie and the transport identity, through `person`'s parse.
    pub fn device_key(&self) -> Result<DevicePublicKey, OrgMembersError> {
        Ok(DevicePublicKey::try_from(self.verifying_key())?)
    }
}

/// A device's ed25519 seed, held only for one operation (LLR-37k2s3): not
/// `Clone`, `Copy`, `Display` or serialisable, and its bytes are zeroed on
/// drop. Its Device public key is derived and parsed once, when the value is
/// built, so `device_key` cannot fail.
pub struct DeviceSecret {
    seed: Zeroizing<[u8; 32]>,
    device_key: DevicePublicKey,
}

impl DeviceSecret {
    /// 32 bytes from the caller's cryptographic random source. A seed whose
    /// verifying key `DevicePublicKey::parse` refused would be drawn again;
    /// the verifying key of an ed25519 seed is a prime-order point, so the
    /// first draw is kept.
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        loop {
            let mut seed = Zeroizing::new([0u8; 32]);
            rng.fill_bytes(seed.as_mut());
            if let Ok(secret) = Self::from_seed(seed) {
                return secret;
            }
        }
    }

    /// LLR-yp69jg: consumes the keychain's buffer, which is zeroed when it
    /// drops here; refuses any length but 32 without echoing a byte.
    pub fn from_keychain_bytes(bytes: Zeroizing<Vec<u8>>) -> Result<Self, OrgNodeError> {
        let mut seed = Zeroizing::new([0u8; 32]);
        if bytes.len() != seed.len() {
            return Err(OrgNodeError::InvalidField {
                field: "Device seed",
                reason: format!("expected 32 bytes, got {}", bytes.len()),
            });
        }
        seed.copy_from_slice(&bytes);
        Self::from_seed(seed)
    }

    /// The ed25519 verifying key of the seed, as a DevicePublicKey.
    pub fn device_key(&self) -> DevicePublicKey {
        self.device_key
    }

    /// LLR-yp69jg: the one way the bytes leave the value. `write` is called
    /// exactly once, and its result is returned: `Ok(())`, or an error bound
    /// to `std::error::Error`, which no byte array implements, so neither
    /// `|s| *s` nor `|s| Err(*s)` compiles (PR-aa7prn, LLR-37k2s3). The bound
    /// does not stop a caller-defined error type that wraps the bytes; that
    /// case rests on review, as LLR-yp69jg's correction of 2026-10-15 says.
    pub fn lend_to_keychain<E: std::error::Error>(
        &self,
        write: impl FnOnce(&[u8; 32]) -> Result<(), E>,
    ) -> Result<(), E> {
        write(&self.seed)
    }
}

impl DeviceSecret {
    /// The seed, inside the crate only: the store key is derived from it
    /// (LLR-2qqqx6). No public operation returns it (LLR-37k2s3).
    #[cfg(feature = "app")]
    pub(crate) fn seed(&self) -> &[u8; 32] {
        &self.seed
    }

    /// The device's signing key pair, built inside the crate for one
    /// signature (an acknowledgement, under `app`) or a test fixture; never
    /// returned by a public operation (LLR-37k2s3).
    #[cfg(any(test, feature = "app", feature = "test-support"))]
    pub(crate) fn signing_keypair(&self) -> SigningKeypair {
        SigningKeypair(SigningKey::from_bytes(&self.seed))
    }

    fn from_seed(seed: Zeroizing<[u8; 32]>) -> Result<Self, OrgNodeError> {
        let verifying_key = SigningKey::from_bytes(&seed).verifying_key();
        let device_key = DevicePublicKey::try_from(verifying_key).map_err(|_| OrgNodeError::InvalidField {
            field: "Device seed",
            reason: "its verifying key is not a valid DevicePublicKey".to_owned(),
        })?;
        Ok(Self { seed, device_key })
    }
}

/// Redacted: the seed never reaches a log.
impl fmt::Debug for DeviceSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DeviceSecret([REDACTED])")
    }
}

/// The seed is `Zeroizing`, which zeroes it on drop.
impl ZeroizeOnDrop for DeviceSecret {}

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
