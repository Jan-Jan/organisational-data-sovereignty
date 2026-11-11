//! Fuzz target: `PersonaStore::unseal` must never panic on arbitrary bytes;
//! an Active store it accepts holds a Persona whose Device key is the one it
//! was given, and what it accepts after the Removed form's index is a
//! removed store (LLR-bg538c). Three shapes: the bytes are the whole sealed store;
//! the bytes are sealed under the right key first, so every iteration reaches
//! the decode and the parse behind the authentication check; and the bytes
//! follow the Removed form's index, so every iteration reaches the removed
//! form's decode and re-parse (LLR-6zaeb6).
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features app,test-support --test fuzz_unseal`;
//! deep-fuzz with `cargo bolero test fuzz_unseal --engine libfuzzer`.
//!
//! verifies: LLR-bg538c, LLR-6zaeb6
#![allow(clippy::expect_used, clippy::panic)]

use bolero::check;
use org_node::keys::DeviceSecret;
use org_node::removal::StoreContent;
use org_node::DevicePublicKey;
use org_node::store::{PersonaStore, SealedStore};
use org_node::test_fixtures::seal_plaintext_for_test;
use rand::rngs::OsRng;
use zeroize::Zeroizing;

fn secret() -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(vec![0x42; 32])).expect("32 bytes")
}

/// An accepted Active store's Persona names `device_key`.
fn assert_active_store_names(store: &PersonaStore, device_key: DevicePublicKey) {
    if let StoreContent::Active(store_data) = store.content() {
        assert_eq!(store_data.persona.device_key, device_key, "the store names the Device it was opened for");
    }
}

fn main() {
    let device_key = secret().device_key();
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the sealed store.
        if let Ok(store) = PersonaStore::unseal(&SealedStore::from_bytes(bytes.to_vec()), device_key, secret()) {
            assert_active_store_names(&store, device_key);
        }
        // Shape 2 — the bytes are the plaintext, sealed under the right key.
        let sealed = seal_plaintext_for_test(&secret(), device_key, bytes, &mut OsRng);
        if let Ok(store) = PersonaStore::unseal(&sealed, device_key, secret()) {
            assert_active_store_names(&store, device_key);
        }
        // Shape 3 — the bytes after the Removed form's index, sealed under the right key: every
        // iteration reaches the removed form's decode and its re-parse (LLR-6zaeb6).
        let mut removed = vec![1u8];
        removed.extend_from_slice(bytes);
        let sealed = seal_plaintext_for_test(&secret(), device_key, &removed, &mut OsRng);
        if let Ok(store) = PersonaStore::unseal(&sealed, device_key, secret()) {
            assert!(matches!(store.content(), StoreContent::Removed(_)), "after the Removed index, a removed store");
        }
    });
}
