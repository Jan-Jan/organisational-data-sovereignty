#![cfg(feature = "write")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The pure decoders behind the signatory-set reads (SDD-rxfu6h).

use on_chain_client::write::AccountId;
use on_chain_client::write::signatory_set::{
    SignatorySetError, decode_original_account, decode_proxy_delegates,
};
use parity_scale_codec::{Compact, Encode};

fn definitions(delegates: &[[u8; 32]]) -> Vec<u8> {
    let mut bytes = Compact(delegates.len() as u32).encode();
    for delegate in delegates {
        bytes.extend_from_slice(delegate);
        bytes.push(0); // ProxyType::Any
        bytes.extend_from_slice(&0u32.to_le_bytes()); // delay
    }
    bytes.extend_from_slice(&1_000u128.to_le_bytes()); // deposit
    bytes
}

// verifies: LLR-m3tjvp
#[test]
fn an_original_account_is_its_32_bytes() {
    assert_eq!(decode_original_account(&[7u8; 32]).unwrap(), AccountId([7u8; 32]));
}

// verifies: LLR-m3tjvp
#[test]
fn an_original_account_of_another_length_is_malformed() {
    assert_eq!(decode_original_account(&[7u8; 31]), Err(SignatorySetError::Malformed));
    assert_eq!(decode_original_account(&[7u8; 33]), Err(SignatorySetError::Malformed));
    assert_eq!(decode_original_account(&[]), Err(SignatorySetError::Malformed));
}

// verifies: LLR-a9bb7b
#[test]
fn proxy_definitions_give_their_delegates_in_order() {
    let bytes = definitions(&[[1u8; 32], [2u8; 32]]);
    assert_eq!(
        decode_proxy_delegates(&bytes).unwrap(),
        vec![AccountId([1u8; 32]), AccountId([2u8; 32])]
    );
    assert_eq!(decode_proxy_delegates(&definitions(&[])).unwrap(), vec![]);
}

// verifies: LLR-a9bb7b
#[test]
fn truncated_padded_or_overlong_definitions_are_malformed() {
    let good = definitions(&[[1u8; 32]]);
    assert_eq!(
        decode_proxy_delegates(&good[..good.len() - 1]),
        Err(SignatorySetError::Malformed)
    );
    let mut padded = good.clone();
    padded.push(0);
    assert_eq!(decode_proxy_delegates(&padded), Err(SignatorySetError::Malformed));
    let mut overlong = Compact(u32::MAX).encode();
    overlong.extend_from_slice(&[0u8; 16]);
    assert_eq!(decode_proxy_delegates(&overlong), Err(SignatorySetError::Malformed));
    assert_eq!(decode_proxy_delegates(&[]), Err(SignatorySetError::Malformed));
}
