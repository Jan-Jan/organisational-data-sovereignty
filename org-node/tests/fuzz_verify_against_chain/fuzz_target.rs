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
//! fuzz bytes as a whole postcard-encoded `SignedDeltaEnvelope`. Almost every
//! such input that decodes at all carries a random `org_id`, so
//! `verify_envelope_against_chain` returned `OrgIdMismatch` at its first check
//! and the remaining seven were never reached: a mutation turning
//! `local_trie.apply_delta(&delta)?` into `.unwrap()` — a panic on any delta
//! that fails to apply — left this target green, and LLR-hs7g6j therefore
//! rested on evidence that only ever exercised step 1. The second shape wraps
//! the fuzz bytes as the **delta** of a correctly signed envelope for the
//! expected Organisation at an acceptable sequence number, so every input
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
use org_node::envelope::SignedDeltaEnvelope;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPublicKey, SequenceNumber};

// The admin's device is a keypair of its own: org-members refuses a leaf whose
// member key is also an enrolled device key (`DuplicateKey`).
fn fixed_trie(admin: &SigningKeypair, admin_device: &SigningKeypair) -> OrgTrie<Blake3Hasher> {
    let leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        admin.member_key(),
        Name::parse("T").unwrap(),
        Surname::parse("U").unwrap(),
        vec![admin_device.device_key()],
    )
    .unwrap();
    OrgTrie::<Blake3Hasher>::genesis(vec![leaf]).unwrap()
}

/// The transcript `SignedDeltaEnvelope::build` signs: org_id ‖ seq (LE) ‖ delta.
/// Rebuilt here because the crate's own `transcript` is private.
fn transcript(org: OrgId, seq: SequenceNumber, delta_bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(20 + 8 + delta_bytes.len());
    buf.extend_from_slice(org.as_bytes());
    buf.extend_from_slice(&seq.get().to_le_bytes());
    buf.extend_from_slice(delta_bytes);
    buf
}

fn main() {
    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    // Same seed as `test_fixtures::admin_device`; this target builds
    // without `test-support`, so it cannot import it.
    let admin_device = DeviceSeed::from([4u8; 32]).signing_keypair();
    let local = fixed_trie(&admin, &admin_device);
    let org = OrgId::new([5u8; 20]);
    let vk = admin.verifying_key();

    // An HONEST delta against `local`, encoded once. Shape 3 perturbs a copy
    // of these bytes, which is the only way to reach the checks past the
    // decode: a delta's `base_root` is thirty-two bytes that must equal the
    // local trie's root, and no generator produces those by chance.
    // Added 2026-10-04 after review round 3 measured that no input reached
    // check 5 — a `panic!()` immediately after the decode left this target
    // passing after 2 479 iterations.
    let joiner = MemberSeed::from([7u8; 32]).signing_keypair();
    let joiner_device = DeviceSeed::from([8u8; 32]).signing_keypair();
    let (honest_root, honest_bytes) = {
        let leaf = MemberLeaf::new(
            MemberId::new([2u8; 32]),
            Handle::parse("joiner").unwrap(),
            joiner.member_key(),
            Name::parse("J").unwrap(),
            Surname::parse("R").unwrap(),
            vec![joiner_device.device_key()],
        )
        .unwrap();
        let (new_trie, delta) =
            local.clone().add_member(leaf).unwrap().recalculate().unwrap();
        (new_trie.root_hash().unwrap(), postcard::to_allocvec(&delta).unwrap())
    };

    // The chain says the Organisation's root is the one the honest delta
    // produces, so an unperturbed shape-3 input is ACCEPTED and the
    // `assert_eq!` below runs; a perturbed one that still applies yields a
    // different root and must be refused at check 8.
    let mut chain = MockChain::new();
    chain.set(
        org,
        OrgState {
            root_hash: honest_root,
            org_pub_key: OrgPublicKey::parse(&[0u8; 32]).unwrap(),
            epoch: Epoch::new(9),
        },
    );

    // bolero wraps each iteration in `catch_unwind`, which requires the
    // closure's captures to be `RefUnwindSafe`. `OrgTrie` contains a
    // `spin::Once` (interior mutability), so wrap the captures.
    // None of local/chain/org/vk can actually be left inconsistent by an unwind
    // because the closure never mutates them — asserting safety is correct.
    let local = AssertUnwindSafe(local);
    let chain = AssertUnwindSafe(chain);

    check!().for_each(move |bytes: &[u8]| {
        let ctx = || VerifyContext {
            expected_org_id: org,
            author_member_key: &vk,
            seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(0)),
            last_committed_epoch: Epoch::new(0),
        };
        let run = |env: &SignedDeltaEnvelope| {
            if let Ok(out) = verify_envelope_against_chain(&*local, env, &ctx(), &*chain) {
                // Any accepted update must equal the chain root it verified against.
                assert_eq!(
                    out.trie.root_hash().unwrap(),
                    chain.get_org_state(&org).unwrap().unwrap().root_hash
                );
            }
        };

        // Shape 1 — the bytes are the whole envelope. Exercises the postcard
        // decode ATTEMPT on `SignedDeltaEnvelope`, which reaches the
        // hand-written `sig_bytes` visitor (a `panic!()` there reddens this
        // target). The `if let Ok(env)` body below is another matter: review
        // round 4 measured that no fuzz input assembles a whole envelope, in
        // this target or in `fuzz_envelope_decode`, so it is never entered.
        // Shapes 2 and 3 build their envelopes rather than hope for one.
        if let Ok(env) = postcard::from_bytes::<SignedDeltaEnvelope>(bytes) {
            run(&env);
        }

        // A well-formed envelope around `delta`: right Organisation, an
        // acceptable sequence number, and a genuine signature over those exact
        // bytes, so checks 1 to 3 always pass and the decode is always
        // reached.
        let seq = SequenceNumber::new(1);
        let envelope_around = |delta: Vec<u8>| SignedDeltaEnvelope {
            org_id: org,
            parent_seq: seq,
            signature: admin.sign(&transcript(org, seq, &delta)).to_bytes(),
            delta_bytes: delta,
        };

        // Shape 2 — the bytes ARE the delta. Reaches the delta decode on every
        // input, which is the untrusted-input surface REQ-bcxz96 is about.
        run(&envelope_around(bytes.to_vec()));

        // Shape 3 — the bytes PERTURB an honest delta: one byte of a valid
        // encoding is overwritten at a fuzz-chosen offset with a fuzz-chosen
        // value. Most results still decode, so the base-root comparison
        // (check 5), `apply_delta` (check 6), the chain read (check 7) and the
        // decisive root match (check 8) are reached — none of which any input
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
