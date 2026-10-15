//! Shared deterministic fixtures for org-node tests. Compiled under `cfg(test)`
//! for the crate's unit tests and under the `test-support` feature for the
//! integration tests in `tests/`; never part of a production build.
//!
//! The `unwrap`/`expect` allow below is deliberate: these are deterministic
//! fixture constructors whose inputs are fixed constants, the module is never
//! part of a production build, and the crate's panic-freedom denial at
//! `lib.rs:2` continues to govern every shipped module.
#![cfg(any(test, feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
use org_members::delta::Delta;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{DevicePublicKey, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, Surname};

use zeroize::Zeroizing;

use crate::keys::{DeviceSecret, SigningKeypair, X25519Keypair};
use crate::types::{MemberSeed, OrgPrivateKey, OrgPublicKey};

pub type Trie = OrgTrie<Blake3Hasher>;

/// Bundles a member's keys + a stable id for building leaves.
pub struct NodeFixture {
    pub member: X25519Keypair,
    pub device: SigningKeypair,
    pub id: MemberId,
}

/// Build a MemberLeaf from a fixture with a fixed handle/name.
pub fn member(fix: &NodeFixture, handle: &str) -> MemberLeaf {
    MemberLeaf::new(
        fix.id,
        Handle::parse(handle).unwrap(),
        fix.member.member_key().unwrap(),
        Name::parse("Test").unwrap(),
        Surname::parse("User").unwrap(),
        vec![fix.device.device_key().unwrap()],
    )
    .unwrap()
}

/// A genesis trie containing a single founding member (id = [1u8;32]). These
/// fixtures call it "admin" (its handle and the names below); it holds no role
/// beyond being the first member — org-node has no administrator.
pub fn genesis_trie(admin: &X25519Keypair, admin_device: &SigningKeypair) -> Trie {
    let admin_fix = NodeFixture {
        member: admin.member_seed().x25519_keypair(),
        device: admin_device.clone(),
        id: MemberId::new([1u8; 32]),
    };
    let leaf = member(&admin_fix, "admin");
    Trie::genesis(vec![leaf]).unwrap()
}

/// Seed of the admin's device keypair. Distinct from every other seed the
/// fixtures use ([1u8;32] admin member, [2u8;32] and [3u8;32] bob), because
/// org-members refuses an organisation in which one key is held twice
/// (`OrgMembersError::DuplicateKey`).
pub const ADMIN_DEVICE_SEED: [u8; 32] = [4u8; 32];

/// The device secret whose seed is `seed`, as org-io's keychain would hand
/// it back (the one form of a device seed, LLR-37k2s3).
pub fn device_secret(seed: [u8; 32]) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(seed.to_vec())).unwrap()
}

/// The device signing key pair whose seed is `seed`, for tests that bind an
/// endpoint or sign as a Device (test builds only; production code builds a
/// key pair only inside the crate, from a `DeviceSecret`).
pub fn device_keypair(seed: [u8; 32]) -> SigningKeypair {
    device_secret(seed).signing_keypair()
}

/// `plaintext` sealed under the store key `secret` derives for
/// `device_key`, exactly as a store seals its data, so a test can present
/// content this software never writes (LLR-8bum44, LLR-bg538c).
#[cfg(feature = "app")]
pub fn seal_plaintext_for_test<R: rand_core::RngCore + rand_core::CryptoRng>(
    secret: &DeviceSecret,
    device_key: DevicePublicKey,
    plaintext: &[u8],
    rng: &mut R,
) -> crate::store::SealedStore {
    let key = crate::store::cipher(secret, &device_key).unwrap();
    crate::store::seal_bytes(&key, plaintext, rng).unwrap()
}

/// An unbound, proposed Persona record named `persona_id`, whose Member seed
/// is 32 bytes of `member` and whose Device is that of device seed 32 bytes
/// of `device`.
#[cfg(feature = "app")]
pub fn persona_record(persona_id: &str, member: u8, device: u8) -> crate::store::PersonaRecord {
    crate::store::PersonaRecord::from(crate::store::PersonaRecordParts {
        persona_id: crate::types::PersonaId::new(persona_id.to_string()),
        org_id: None,
        handle: Handle::parse("alice").unwrap(),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        member_seed: MemberSeed::from([member; 32]),
        device_key: device_key(device),
        member_id: None,
        status: crate::store::PersonaStatus::Proposed,
    })
}

/// A store holding `persona` and nothing else.
#[cfg(feature = "app")]
pub fn empty_store(persona: crate::store::PersonaRecord) -> crate::store::StoreData {
    crate::store::StoreData {
        persona,
        orgs: Vec::new(),
        provisional_updates: Vec::new(),
        expected_admissions: Vec::new(),
        outstanding_invites: Vec::new(),
    }
}

/// An open store over `data`, keyed from `secret`, whose Device must be
/// `data`'s Persona's: what `PersonaStore::unseal` returns for `data`, built
/// without sealing it first.
#[cfg(feature = "app")]
pub fn store_for_test(secret: &DeviceSecret, data: crate::store::StoreData) -> crate::store::PersonaStore {
    let mut store = crate::store::PersonaStore::create(secret, data.persona.clone()).unwrap();
    *store.data_mut() = data;
    store
}

/// `store`, sealed as an operation that changed it would seal it.
#[cfg(feature = "app")]
pub fn seal_for_test<R: rand_core::RngCore + rand_core::CryptoRng>(
    store: &crate::store::PersonaStore,
    rng: &mut R,
) -> crate::store::StoreWrite {
    store.seal(rng).unwrap()
}

/// The admin's device keypair: a key of its own, never the admin's member key.
pub fn admin_device() -> SigningKeypair {
    device_keypair(ADMIN_DEVICE_SEED)
}

/// Seed of bob's device keypair in [`admit_member_delta`].
pub const BOB_DEVICE_SEED: [u8; 32] = [3u8; 32];

/// Bob's device keypair: the device [`admit_member_delta`] enrols.
pub fn bob_device() -> SigningKeypair {
    device_keypair(BOB_DEVICE_SEED)
}

/// Build the "admit member B (id=[2u8;32])" delta against a genesis trie
/// whose admin member key is `admin` and whose admin device is
/// [`admin_device`]. Returns (delta, new_trie).
pub fn admit_member_delta(admin: &X25519Keypair) -> (Delta, Trie) {
    let base = genesis_trie(admin, &admin_device());
    let b_fix = NodeFixture {
        member: MemberSeed::from([2u8; 32]).x25519_keypair(),
        device: bob_device(),
        id: MemberId::new([2u8; 32]),
    };
    let leaf = member(&b_fix, "bob");
    let (new_trie, delta) = base.add_member(leaf).unwrap().recalculate().unwrap();
    (delta, new_trie)
}

/// A valid Organisation public key, for chain states in tests.
pub fn org_public_key() -> OrgPublicKey {
    OrgPrivateKey::from([9u8; 32]).x25519_keypair().org_public_key().unwrap()
}

/// The Member-as-a-group key of the X25519 key pair whose member seed is 32
/// bytes of `seed`.
pub fn member_key(seed: u8) -> PersonPublicKey {
    MemberSeed::from([seed; 32]).x25519_keypair().member_key().unwrap()
}

/// The DevicePublicKey of the key pair whose device seed is 32 bytes of `seed`.
pub fn device_key(seed: u8) -> DevicePublicKey {
    device_secret([seed; 32]).device_key()
}

/// A revocation notice for `org_id` naming Member `[2u8;32]` on Bob's
/// Device, which the genesis trie does not list, with a proof of its absence
/// from that trie.
pub fn bob_absence_notice(org_id: crate::ids::OrgId) -> crate::revocation::RevocationNotice {
    let trie = genesis_trie(&MemberSeed::from([1u8; 32]).x25519_keypair(), &admin_device());
    let member_id = MemberId::new([2u8; 32]);
    let device = bob_device().device_key().unwrap();
    let proof = trie.prove_absent(&member_id, &device).unwrap();
    crate::revocation::RevocationNotice { org_id, member_id, device, proof }
}

/// The slots and the id counter `ChainSlots` shares between its clones.
#[derive(Default)]
struct ChainSlotsInner {
    slots: std::collections::HashMap<crate::ids::OrgId, crate::chain::OrgState>,
    next_id_seed: u8,
}

/// A test-side store of Organisation states, standing in for the chain a
/// test's org-io would read: org-node's chain-judging operations take the
/// state as a value (ruling B), and a test passes them `get(&org_id)`.
/// Clones share one set of slots, so two services under test observe the
/// same chain (LLR-hg3xzf).
#[derive(Clone, Default)]
pub struct ChainSlots {
    inner: std::sync::Arc<std::sync::Mutex<ChainSlotsInner>>,
}

impl ChainSlots {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ChainSlotsInner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Set a slot directly (for test setup only).
    pub fn set(&self, org_id: crate::ids::OrgId, state: crate::chain::OrgState) {
        self.lock().slots.insert(org_id, state);
    }

    /// The state of a slot, `None` for an Organisation with none.
    pub fn get(&self, org_id: &crate::ids::OrgId) -> Option<crate::chain::OrgState> {
        self.lock().slots.get(org_id).copied()
    }

    /// Stand-in for the genesis chain write org-io makes through
    /// on-chain-client: a slot at epoch one holding `root` and `key`, under an
    /// id derived from the root and a counter (LLR-ryzr8m).
    pub fn apply_genesis(&self, root: org_members::RootHash, key: OrgPublicKey) -> crate::ids::OrgId {
        let mut inner = self.lock();
        let mut id_bytes = [0u8; 20];
        id_bytes.copy_from_slice(&root.as_bytes()[..20]);
        id_bytes[0] ^= inner.next_id_seed;
        inner.next_id_seed = inner.next_id_seed.wrapping_add(1);
        let org_id = crate::ids::OrgId::new(id_bytes);
        inner.slots.insert(
            org_id,
            crate::chain::OrgState { root_hash: root, org_pub_key: key, epoch: crate::types::Epoch::new(1) },
        );
        org_id
    }

    /// Stand-in for the update chain write org-io makes through
    /// on-chain-client: `root` and `key` at `expected_epoch + 1`, refused
    /// unless the slot is at `expected_epoch` (LLR-ryzr8m).
    pub fn apply_update(
        &self,
        org_id: crate::ids::OrgId,
        root: org_members::RootHash,
        key: OrgPublicKey,
        expected_epoch: crate::types::Epoch,
    ) -> Result<(), crate::error::OrgNodeError> {
        let mut inner = self.lock();
        let state = inner.slots.get(&org_id).copied().ok_or(crate::error::OrgNodeError::OrgNotOnChain)?;
        if state.epoch != expected_epoch {
            return Err(crate::error::OrgNodeError::Chain(format!(
                "epoch mismatch: expected {}, found {}",
                expected_epoch.get(),
                state.epoch.get()
            )));
        }
        inner.slots.insert(
            org_id,
            crate::chain::OrgState {
                root_hash: root,
                org_pub_key: key,
                epoch: crate::types::Epoch::new(expected_epoch.get() + 1),
            },
        );
        Ok(())
    }
}

/// Answers at run time whether a type implements `Display` or `Copy`, so a
/// test can assert that a secret type implements neither (LLR-sz4xhc,
/// LLR-scgk5j). Autoref specialisation: the impl on `Probe<T>` applies when
/// its bound holds, otherwise method lookup falls through to the impl on
/// `&Probe<T>`. Use through `implements_display!` / `implements_copy!`.
pub struct Probe<T>(core::marker::PhantomData<T>);

impl<T> Probe<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

pub trait ImplementsDisplay {
    fn implements_display(&self) -> bool {
        true
    }
}
impl<T: core::fmt::Display> ImplementsDisplay for Probe<T> {}

pub trait LacksDisplay {
    fn implements_display(&self) -> bool {
        false
    }
}
impl<T> LacksDisplay for &Probe<T> {}

pub trait ImplementsCopy {
    fn implements_copy(&self) -> bool {
        true
    }
}
impl<T: Copy> ImplementsCopy for Probe<T> {}

pub trait LacksCopy {
    fn implements_copy(&self) -> bool {
        false
    }
}
impl<T> LacksCopy for &Probe<T> {}

/// `implements_display!(T)`: does `T` implement `Display`?
#[macro_export]
macro_rules! implements_display {
    ($t:ty) => {{
        #[allow(unused_imports)]
        use $crate::test_fixtures::{ImplementsDisplay as _, LacksDisplay as _};
        (&$crate::test_fixtures::Probe::<$t>::new()).implements_display()
    }};
}

/// `implements_copy!(T)`: does `T` implement `Copy`?
#[macro_export]
macro_rules! implements_copy {
    ($t:ty) => {{
        #[allow(unused_imports)]
        use $crate::test_fixtures::{ImplementsCopy as _, LacksCopy as _};
        (&$crate::test_fixtures::Probe::<$t>::new()).implements_copy()
    }};
}
