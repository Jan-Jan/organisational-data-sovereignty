//! Fuzz target: the record a first admission extends.
//!
//! **Two shapes per input.** Shape 1 offers the fuzz bytes as a
//! postcard-encoded `Vec<MemberSnapshot>` and calls `first_admission_base`
//! on every input; almost every input that decodes at all decodes to the
//! empty vector, so shape 1 alone does not reach the key parse.
//!
//! Shape 2 builds a **well-formed** `Vec<MemberSnapshot>` whose key bytes
//! come from the fuzz input, so the key parse — the snapshot's decode into
//! its typed record, which refuses with
//! `InvalidField { field: "member.member_key" | "member.device_keys", .. }`
//! (LLR-7yu2xy) — is reached on every iteration. Those are the
//! abnormal-input arms SDD-rx2yvy's robustness entry names.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run a single target with
//! `cargo test -p org-node --features app --test fuzz_first_admission_base`;
//! deep-fuzz with
//! `cargo bolero test fuzz_first_admission_base --engine libfuzzer`.

use org_node::service::first_admission_base;
use serde::Serialize;

/// A member snapshot as it is encoded, with its fields plain.
/// `store::MemberSnapshot` holds parsed keys (LLR-pv2rdy), so it cannot carry
/// the arbitrary key bytes shape 2 exists to deliver. This mirror has the same field order and
/// plain types, so it encodes to the bytes a `MemberSnapshot` would
/// (LLR-mkvx2k), and `first_admission_base` parses them through its own
/// `Raw…` mirror (LLR-7yu2xy).
#[derive(Serialize)]
struct PlainSnapshot {
    id: [u8; 32],
    handle: String,
    name: String,
    surname: String,
    member_key: [u8; 32],
    device_keys: Vec<[u8; 32]>,
}

/// One snapshot whose id, member key and device key are taken from `bytes`,
/// cycling so that any non-empty input fills all three.
fn snapshot_from(bytes: &[u8]) -> PlainSnapshot {
    let take = |offset: usize| {
        let mut out = [0u8; 32];
        for (index, slot) in out.iter_mut().enumerate() {
            *slot = bytes[(offset + index) % bytes.len()];
        }
        out
    };
    PlainSnapshot {
        id: take(0),
        handle: "fuzz".into(),
        name: "Fuzz".into(),
        surname: "Target".into(),
        member_key: take(32),
        device_keys: vec![take(64)],
    }
}

// Abnormal case of REQ-d9g6nt over arbitrary input: arbitrary snapshot bytes
// never panic. (A first admission without a snapshot cannot be expressed:
// Organisation information requires one, LLR-xr88a8, LLR-j6j95z.)
// verifies: REQ-d9g6nt
fn main() {
    bolero::check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the whole encoded snapshot vector. Exercises
        // postcard's decode of `Vec<MemberSnapshot>`, including its length
        // prefix and the `String` fields.
        let _ = first_admission_base(bytes);

        // Shape 2 — the bytes are the KEY MATERIAL inside a well-formed
        // snapshot vector, so every iteration reaches `VerifyingKey::from_bytes`
        // on a member key and on a device key. Most 32-byte strings are not
        // valid ed25519 points; those must come back as a typed error, never a
        // panic.
        if !bytes.is_empty() {
            let snaps = vec![snapshot_from(bytes)];
            if let Ok(encoded) = postcard::to_allocvec(&snaps) {
                let _ = first_admission_base(&encoded);
            }
        }
    });
}
