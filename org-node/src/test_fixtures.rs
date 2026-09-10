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
use org_members::{MemberId, MemberLeaf};

use crate::keys::SigningKeypair;

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
        handle,
        fix.keypair.member_key(),
        "Test",
        "User",
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
    let (trie, _delta) = Trie::genesis(vec![leaf]).unwrap().recalculate().unwrap();
    trie
}

/// Build the "admit member B (id=[2u8;32])" delta against a genesis trie
/// authored by `admin`. Returns (delta, new_trie). `admin` doubles as the
/// admin device for fixture simplicity.
pub fn admit_member_delta(admin: &SigningKeypair) -> (Delta, Trie) {
    let base = genesis_trie(admin, admin);
    let b_member = SigningKeypair::from_seed([2u8; 32]);
    let b_device = SigningKeypair::from_seed([3u8; 32]);
    let b_fix = NodeFixture { keypair: b_member, device: b_device, id: MemberId::new([2u8; 32]) };
    let leaf = member(&b_fix, "bob");
    let (new_trie, delta) = base.add_member(leaf).unwrap().recalculate().unwrap();
    (delta, new_trie)
}

