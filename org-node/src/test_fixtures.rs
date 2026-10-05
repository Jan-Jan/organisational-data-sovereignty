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
use org_members::{Handle, MemberId, MemberLeaf, Name, P2pDeviceKey, P2pMemberKey, Surname};

use crate::keys::SigningKeypair;
use crate::types::{DeviceSeed, MemberSeed};

pub type Trie = OrgTrie<Blake3Hasher>;

/// Bundles a member's keys + a stable id for building leaves.
pub struct NodeFixture {
    pub keypair: SigningKeypair,
    pub device: SigningKeypair,
    pub id: MemberId,
}

/// Build a MemberLeaf from a fixture with a fixed handle/name.
pub fn member(fix: &NodeFixture, handle: &str) -> MemberLeaf {
    MemberLeaf::new(
        fix.id,
        Handle::parse(handle).unwrap(),
        fix.keypair.member_key(),
        Name::parse("Test").unwrap(),
        Surname::parse("User").unwrap(),
        vec![fix.device.device_key()],
    )
    .unwrap()
}

/// A genesis trie containing a single admin member (id = [1u8;32]).
pub fn genesis_trie(admin: &SigningKeypair, admin_device: &SigningKeypair) -> Trie {
    let admin_fix = NodeFixture {
        keypair: admin.clone(),
        device: admin_device.clone(),
        id: MemberId::new([1u8; 32]),
    };
    let leaf = member(&admin_fix, "admin");
    Trie::genesis(vec![leaf]).unwrap()
}

/// The admin's device keypair: a key of its own, never the admin's member key.
/// Its seed ([4u8;32]) is distinct from every other seed the fixtures use
/// ([1u8;32] admin member, [2u8;32] and [3u8;32] bob), because org-members
/// refuses an organisation in which one key is held twice
/// (`OrgMembersError::DuplicateKey`), a member key equal to its own device key
/// included.
pub fn admin_device() -> SigningKeypair {
    DeviceSeed::from([4u8; 32]).signing_keypair()
}

/// Build the "admit member B (id=[2u8;32])" delta against a genesis trie
/// authored by `admin`, whose device is [`admin_device`]. Returns
/// (delta, new_trie).
pub fn admit_member_delta(admin: &SigningKeypair) -> (Delta, Trie) {
    let base = genesis_trie(admin, &admin_device());
    let b_member = MemberSeed::from([2u8; 32]).signing_keypair();
    let b_device = DeviceSeed::from([3u8; 32]).signing_keypair();
    let b_fix = NodeFixture { keypair: b_member, device: b_device, id: MemberId::new([2u8; 32]) };
    let leaf = member(&b_fix, "bob");
    let (new_trie, delta) = base.add_member(leaf).unwrap().recalculate().unwrap();
    (delta, new_trie)
}

/// The Member key of the key pair whose seed is 32 bytes of `seed`.
pub fn member_key(seed: u8) -> P2pMemberKey {
    MemberSeed::from([seed; 32]).signing_keypair().member_key()
}

/// The Device key of the key pair whose seed is 32 bytes of `seed`.
pub fn device_key(seed: u8) -> P2pDeviceKey {
    DeviceSeed::from([seed; 32]).signing_keypair().device_key()
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
