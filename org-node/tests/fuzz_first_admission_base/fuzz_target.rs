//! Fuzz target: the record a first admission extends.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run a single target with
//! `cargo test -p org-node --features app --test fuzz_first_admission_base`;
//! deep-fuzz with
//! `cargo bolero test fuzz_first_admission_base --engine libfuzzer`.

use org_node::OrgNodeError;
use org_node::service::first_admission_base;

// Abnormal case of REQ-d9g6nt over arbitrary input: no snapshot is
// always refused, and arbitrary snapshot bytes never panic.
// verifies: REQ-d9g6nt
fn main() {
    // No snapshot: refused with its own error, not some later failure.
    match first_admission_base(None) {
        Err(OrgNodeError::Chain(msg))
            if msg.contains("first admission without a record snapshot") => {}
        Err(other) => {
            panic!("first admission without a snapshot refused for the wrong reason: {other:?}")
        }
        Ok(_) => panic!("first admission without a snapshot was accepted"),
    }
    bolero::check!().for_each(|bytes: &[u8]| {
        let _ = first_admission_base(Some(bytes));
    });
}
