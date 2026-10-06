//! Fuzz target: `verify_envelope_against_chain` must never panic on a malformed
//! envelope, and any accepted update must match the on-chain root.
//!
//! **The chain holds the root an honest update produces, not the local root.**
//! Corrected 2026-10-04 after review round 4. The chain used to be seeded with
//! `local.root_hash()` — the root *before* any delta — so check 8 compared a
//! post-apply root against a pre-apply one and failed for every delta that
//! changed the trie: **no input could be accepted, by construction**, and the
//! `assert_eq!` on an accepted update, this target's second property, could
//! never run. A `panic!()` placed after check 8 succeeded left the target
//! green. Reaching check 8 was not passing it.
//!
//! The fixed, honestly-built local trie + chain are the honest context; only
//! the envelope is fuzz-controlled.
//!
//! **Two shapes per input, added 2026-10-04.** The original target offered the
//! fuzz bytes as a whole postcard-encoded envelope (then `SignedDeltaEnvelope`). Almost every
//! such input that decodes at all carries a random `org_id`, so
//! `verify_envelope_against_chain` returned `OrgIdMismatch` at its first check
//! and the remaining seven were never reached: a mutation turning
//! `local_trie.apply_delta(&delta)?` into `.unwrap()` — a panic on any delta
//! that fails to apply — left this target green, and LLR-hs7g6j therefore
//! rested on evidence that only ever exercised step 1. The second shape wraps
//! the fuzz bytes as the **delta** of a well-formed envelope for the
//! expected Organisation at an acceptable sequence number (the envelope
//! carries no signature and its sender is not checked since 2026-10-05,
//! REQ-ag6kqm), so every input
//! reaches the decode, and a decodable-but-inapplicable delta reaches the
//! base-root check and `apply_delta`.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run with
//! `cargo test -p org-node --test fuzz_verify_against_chain`; deep-fuzz with
//! `cargo bolero test fuzz_verify_against_chain --engine libfuzzer`.
//!
//! verifies: REQ-bcxz96, LLR-hs7g6j

use std::panic::AssertUnwindSafe;

use bolero::check;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};
use org_node::chain::{ChainReader, MockChain, OrgState};
use org_node::envelope::Envelope;
use org_node::ids::OrgId;
use org_node::keys::{SigningKeypair, X25519Keypair};
use org_node::sequence::SeqGuard;
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, SequenceNumber};

// The admin's device is a keypair of its own: org-members refuses a leaf whose
// member key is also an enrolled device key (`DuplicateKey`).
fn fixed_trie(admin: &X25519Keypair, admin_device: &SigningKeypair) -> OrgTrie<Blake3Hasher> {
    let leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        admin.member_key().expect("valid key"),
        Name::parse("T").unwrap(),
        Surname::parse("U").unwrap(),
        vec![admin_device.device_key().unwrap()],
    )
    .unwrap();
    OrgTrie::<Blake3Hasher>::genesis(vec![leaf]).unwrap()
}

fn main() {
    // Same seed as `test_fixtures::ADMIN_DEVICE_SEED`; this target builds
    // without `test-support`, so it cannot import it.
    let admin_device = DeviceSeed::from([4u8; 32]).signing_keypair();
    let member = MemberSeed::from([1u8; 32]).x25519_keypair();
    let local = fixed_trie(&member, &admin_device);
    let org = OrgId::new([5u8; 20]);
    // An HONEST delta against `local`, encoded once. Shape 3 perturbs a copy
    // of these bytes, which is the only way to reach the checks past the
    // decode: a delta's `base_root` is thirty-two bytes that must equal the
    // local trie's root, and no generator produces those by chance.
    // Added 2026-10-04 after review round 3 measured that no input reached
    // check 5 — a `panic!()` immediately after the decode left this target
    // passing after 2 479 iterations.
    let joiner = MemberSeed::from([7u8; 32]).x25519_keypair();
    let joiner_device = DeviceSeed::from([8u8; 32]).signing_keypair();
    let (honest_root, honest_bytes) = {
        let leaf = MemberLeaf::new(
            MemberId::new([2u8; 32]),
            Handle::parse("joiner").unwrap(),
            joiner.member_key().unwrap(),
            Name::parse("J").unwrap(),
            Surname::parse("R").unwrap(),
            vec![joiner_device.device_key().unwrap()],
        )
        .unwrap();
        let (new_trie, delta) =
            local.clone().add_member(leaf).unwrap().recalculate().unwrap();
        (new_trie.root_hash().unwrap(), postcard::to_allocvec(&delta).unwrap())
    };

    // The chain says the Organisation's root is the one the honest delta
    // produces, so an unperturbed shape-3 input is ACCEPTED and the
    // `assert_eq!` below runs; a perturbed one that still applies yields a
    // different root and must be refused at the root match (check 7).
    let mut chain = MockChain::new();
    chain.set(
        org,
        OrgState {
            root_hash: honest_root,
            org_pub_key: OrgPrivateKey::from([9u8; 32]).x25519_keypair().org_public_key().unwrap(),
            epoch: Epoch::new(9),
        },
    );

    // bolero wraps each iteration in `catch_unwind`, which requires the
    // closure's captures to be `RefUnwindSafe`. `OrgTrie` contains a
    // `spin::Once` (interior mutability), so wrap the captures.
    // None of local/chain/org can actually be left inconsistent by an unwind
    // because the closure never mutates them — asserting safety is correct.
    let local = AssertUnwindSafe(local);
    let chain = AssertUnwindSafe(chain);

    check!().for_each(move |bytes: &[u8]| {
        let ctx = || VerifyContext {
            expected_org_id: org,
            seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(0)),
            last_committed_epoch: Epoch::new(0),
        };
        let run = |env: &Envelope| {
            if let Ok(out) = verify_envelope_against_chain(&*local, env, &ctx(), &*chain) {
                // Any accepted update must equal the chain root it verified against.
                assert_eq!(
                    out.trie.root_hash().unwrap(),
                    chain.get_org_state(&org).unwrap().unwrap().root_hash
                );
            }
        };

        // Shape 1 — the bytes are the whole envelope: the postcard decode
        // ATTEMPT on `Envelope`. Review round 4 measured (on the signed
        // envelope of the time) that no fuzz input assembled a whole
        // envelope; the unsigned `Envelope` needs only 22 bytes, and whether
        // the body is now entered has not been re-measured. Shapes 2 and 3
        // build their envelopes rather than hope for one.
        if let Ok(env) = postcard::from_bytes::<Envelope>(bytes) {
            run(&env);
        }

        // A well-formed envelope around `delta`: right Organisation and an
        // acceptable sequence number, so checks 1 and 2 always pass and the
        // decode is always reached. The number is the
        // chain's epoch (9), as REQ-txvtm9 requires since 2026-10-05, so the
        // unperturbed shape-3 input is still accepted.
        let seq = SequenceNumber::new(9);
        let envelope_around =
            |delta: Vec<u8>| Envelope { org_id: org, parent_seq: seq, delta_bytes: delta };

        // Shape 2 — the bytes ARE the delta. Reaches the delta decode on every
        // input, which is the untrusted-input surface REQ-bcxz96 is about.
        run(&envelope_around(bytes.to_vec()));

        // Shape 3 — the bytes PERTURB an honest delta: one byte of a valid
        // encoding is overwritten at a fuzz-chosen offset with a fuzz-chosen
        // value. Most results still decode, so the base-root comparison
        // (check 4), `apply_delta` (check 5), the chain read (check 6) and the
        // decisive root match (check 7) are reached — none of which any input
        // reached before this shape existed. An unperturbed delta — the chosen
        // value happens to match what was there, one input in 256 — is
        // accepted, because the chain holds the honest root; that is the path
        // on which the `assert_eq!` runs.
        if bytes.len() >= 2 && !honest_bytes.is_empty() {
            let mut perturbed = honest_bytes.clone();
            let at = (bytes[0] as usize) % perturbed.len();
            perturbed[at] = bytes[1];
            run(&envelope_around(perturbed));
        }
    });
}
