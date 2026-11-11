//! Fuzz target: `OrgService::check_received` never panics on any body, and a
//! body it refuses leaves the receiving store as it was (LLR-7u3dg7). It
//! carries forward the deleted `fuzz_verify_against_chain`'s shapes 2 and 3,
//! re-targeted at receipt, where a received Change set is now applied
//! (merge review MR2-N1).
//!
//! **Two receiving stores.** One holds a current record (Ada's, epoch 2,
//! listing Ada and Bob) and is offered bodies from Bob's listed Device, so
//! Organisation information reaches `keep_information` past the sender rule.
//! The other holds no record (Carol's provisional Member, expecting
//! admission to the Organisation) and is offered bodies from a Device no
//! record lists, which `check_without_record` accepts Organisation
//! information from for an expected Organisation.
//!
//! **Four shapes per input, each offered to both stores.** Shape 1: the
//! bytes are the whole body. Shape 2: the bytes are the Change set of an
//! honest Organisation-information body for that store, so every input
//! reaches the Change-set decode. Shape 3: one byte of the honest Change set
//! is overwritten at a fuzz-chosen offset with a fuzz-chosen value; most
//! results still decode, so the base-root check, `apply_delta` and the root
//! computation are reached. Shape 4: one byte of the honest encoded body is
//! overwritten the same way. The offset is taken from two bytes, modulo the
//! length perturbed, so every byte of the honest Change set (152 bytes) and
//! of each honest body (437 bytes with a record, 323 with none) can be
//! chosen: shape 4 reaches the record snapshot's decode and the carried
//! Organisation private key's decode as well as the header's (merge review
//! MR3, finding-2).
//!
//! **What "unchanged" compares.** The store's encoded bytes and its behind
//! marker, which the encoding leaves out: a refusal that set the marker
//! would change what the next operation on the store does.
//!
//! **What this target does not reach.** Neither store has its behind marker
//! set, so the organisation answer's snapshot decode is not reached through
//! `check_received`; `fuzz_first_admission_base` covers that decode
//! directly.
//!
//! **Iterations.** The gated run is bolero's default one-second budget,
//! about 50 inputs (eight offers each). That is left as it is: the gate
//! shows the target runs and holds on a fresh random sample each run, and a
//! deep run is `cargo bolero test` below. Nothing keeps a corpus between
//! runs.
//!
//! A body that is kept changes the store; the offer after it starts from a
//! freshly built store, so every offer starts from the same store.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features app,test-support --test fuzz_check_received`;
//! deep-fuzz with `cargo bolero test fuzz_check_received --engine libfuzzer`.
//!
//! verifies: LLR-7u3dg7, REQ-rv4dwb
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "../support/mod.rs"]
mod support;

use std::cell::RefCell;
use std::panic::AssertUnwindSafe;

use bolero::check;
use org_node::service::OrgService;
use org_node::test_fixtures::device_key;
use org_node::wire::{encode_message, WireMessage};
use org_node::{DevicePublicKey, Epoch};
use rand::rngs::OsRng;
use support::fixtures::{ada, ada_store, bob, provisional_member, record, service, ADA, BOB, EVE};
use support::{encoded_store, envelope_of, held_store, information_after, with_envelope};

/// One receiving store: how to build it, the store as built, the Device its
/// bodies come from, and an honest Organisation-information body for it.
struct Receiver {
    build: fn() -> (OrgService, DevicePublicKey),
    held: RefCell<(OrgService, DevicePublicKey)>,
    sender: DevicePublicKey,
    honest: WireMessage,
}

/// Ada's store holding the record at epoch 2 of Ada and Bob.
fn with_record() -> (OrgService, DevicePublicKey) {
    (service(ADA, ada_store(2, &[ada(), bob()])), device_key(ADA))
}

impl Receiver {
    fn new(build: fn() -> (OrgService, DevicePublicKey), sender: DevicePublicKey, honest: WireMessage) -> Self {
        Receiver { build, held: RefCell::new(build()), sender, honest }
    }

    /// Offer `body`: no panic, and a refusal leaves the store's bytes and its
    /// behind marker as they were. A body that changed the store is followed
    /// by a freshly built one, so every offer starts from the same store.
    fn offer(&self, body: &[u8]) {
        let mut held = self.held.borrow_mut();
        let (svc, device) = &mut *held;
        let before = store_state(svc, device);
        let refused = svc.check_received(&mut OsRng, device, self.sender, body).is_err();
        let after = store_state(svc, device);
        if refused {
            assert_eq!(after, before, "a refused body changes no store");
        } else if after != before {
            *held = (self.build)();
        }
    }

    /// The honest body with `delta_bytes` as its Change set.
    fn carrying(&self, delta_bytes: Vec<u8>) -> Vec<u8> {
        let envelope = org_node::Envelope { delta_bytes, ..envelope_of(&self.honest).clone() };
        encode_message(&with_envelope(&self.honest, envelope)).unwrap()
    }
}

/// The store `device` names as a refusal must leave it: its encoded bytes
/// and its behind marker, which the encoding leaves out.
fn store_state(svc: &OrgService, device: &DevicePublicKey) -> (Vec<u8>, Option<Epoch>) {
    (encoded_store(svc, device), held_store(svc, device).behind)
}

/// `honest` with the byte at the offset `bytes[0..2]` chooses (big-endian,
/// modulo `honest`'s length, so any byte of a body up to 64 KiB) set to
/// `bytes[2]`.
fn perturbed(honest: &[u8], bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 3 || honest.is_empty() {
        return None;
    }
    let mut perturbed = honest.to_vec();
    let at = usize::from(u16::from_be_bytes([bytes[0], bytes[1]])) % perturbed.len();
    perturbed[at] = bytes[2];
    Some(perturbed)
}

fn main() {
    let receivers = [
        Receiver::new(with_record, device_key(BOB), information_after(&record(2, &[ada(), bob()]), 0x70).0),
        Receiver::new(provisional_member, device_key(EVE), information_after(&record(1, &[ada()]), 0x72).0),
    ];
    // A panic fails the run, so no iteration follows an unwind that could
    // have left a held store half-changed.
    let receivers = AssertUnwindSafe(receivers);

    check!().for_each(move |bytes: &[u8]| {
        for receiver in receivers.iter() {
            // Shape 1 — the bytes are the whole body.
            receiver.offer(bytes);
            // Shape 2 — the bytes are the Change set.
            receiver.offer(&receiver.carrying(bytes.to_vec()));
            // Shape 3 — one byte of the honest Change set perturbed.
            if let Some(delta) = perturbed(&envelope_of(&receiver.honest).delta_bytes, bytes) {
                receiver.offer(&receiver.carrying(delta));
            }
            // Shape 4 — one byte of the honest body perturbed.
            if let Some(body) = perturbed(&encode_message(&receiver.honest).unwrap(), bytes) {
                receiver.offer(&body);
            }
        }
    });
}
