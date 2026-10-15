//! Fuzz target: `PersonaStore::unseal` must never panic on arbitrary bytes,
//! and whatever it accepts is a store whose Device key is the one it was
//! given (LLR-bg538c). Two shapes: the bytes are the whole sealed store; and
//! the bytes are sealed under the right key first, so every iteration reaches
//! the decode and the parse behind the authentication check.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features app,test-support --test fuzz_unseal`;
//! deep-fuzz with `cargo bolero test fuzz_unseal --engine libfuzzer`.
//!
//! verifies: LLR-bg538c
#![allow(clippy::expect_used, clippy::panic)]

use bolero::check;
use org_node::keys::DeviceSecret;
use org_node::store::{PersonaStore, SealedStore};
use org_node::test_fixtures::seal_plaintext_for_test;
use rand::rngs::OsRng;
use zeroize::Zeroizing;

fn secret() -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(vec![0x42; 32])).expect("32 bytes")
}

fn main() {
    let device_key = secret().device_key();
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the sealed store.
        if let Ok(store) = PersonaStore::unseal(&SealedStore::from_bytes(bytes.to_vec()), device_key, secret()) {
            assert_eq!(store.device_key(), device_key);
        }
        // Shape 2 — the bytes are the plaintext, sealed under the right key.
        let sealed = seal_plaintext_for_test(&secret(), device_key, bytes, &mut OsRng);
        if let Ok(store) = PersonaStore::unseal(&sealed, device_key, secret()) {
            assert_eq!(store.device_key(), device_key);
        }
    });
}
