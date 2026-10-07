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

use crate::keys::{SigningKeypair, X25519Keypair};
use crate::types::{DeviceSeed, MemberSeed, OrgPrivateKey, OrgPublicKey};

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

/// The admin's device keypair: a key of its own, never the admin's member key.
pub fn admin_device() -> SigningKeypair {
    DeviceSeed::from(ADMIN_DEVICE_SEED).signing_keypair()
}

/// Seed of bob's device keypair in [`admit_member_delta`].
pub const BOB_DEVICE_SEED: [u8; 32] = [3u8; 32];

/// Bob's device keypair: the device [`admit_member_delta`] enrols.
pub fn bob_device() -> SigningKeypair {
    DeviceSeed::from(BOB_DEVICE_SEED).signing_keypair()
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
    DeviceSeed::from([seed; 32]).signing_keypair().device_key().unwrap()
}

/// A revocation notice for `org_id` naming Member `[2u8;32]` on Bob's
/// Device, which the genesis trie does not list, with a proof of its absence
/// from that trie.
#[cfg(feature = "transport")]
pub fn bob_absence_notice(org_id: crate::ids::OrgId) -> crate::revocation::RevocationNotice {
    let trie = genesis_trie(&MemberSeed::from([1u8; 32]).x25519_keypair(), &admin_device());
    let member_id = MemberId::new([2u8; 32]);
    let device = bob_device().device_key().unwrap();
    let proof = trie.prove_absent(&member_id, &device).unwrap();
    crate::revocation::RevocationNotice { org_id, member_id, device, proof }
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
