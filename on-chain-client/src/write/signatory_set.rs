//! The signatory-set reads' pure half (SDD-rxfu6h): decoding the stored
//! values of `Revive.OriginalAccount` and `Proxy.Proxies`. The fetches that
//! hand these decoders their bytes are in `client.rs`.

use parity_scale_codec::{Compact, Decode};

use super::AccountId;

/// A stored value that is not what the runtime stores there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatorySetError {
    Malformed,
}

impl core::fmt::Display for SignatorySetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SignatorySetError::Malformed => f.write_str("malformed signatory-set storage value"),
        }
    }
}

impl std::error::Error for SignatorySetError {}

/// One proxy definition: a 32-byte delegate, a one-byte proxy type and a
/// four-byte delay.
const DEFINITION_BYTES: usize = 32 + 1 + 4;
/// The `u128` deposit that follows the definitions.
const DEPOSIT_BYTES: usize = 16;

/// `Revive.OriginalAccount`'s value: an AccountId32 (LLR-m3tjvp).
pub fn decode_original_account(bytes: &[u8]) -> Result<AccountId, SignatorySetError> {
    <[u8; 32]>::try_from(bytes)
        .map(AccountId)
        .map_err(|_| SignatorySetError::Malformed)
}

/// `Proxy.Proxies`' value: the delegates, in stored order (LLR-a9bb7b).
/// The length prefix is checked against the bytes that follow before
/// anything is allocated.
pub fn decode_proxy_delegates(bytes: &[u8]) -> Result<Vec<AccountId>, SignatorySetError> {
    let mut input = bytes;
    let count = Compact::<u32>::decode(&mut input)
        .map_err(|_| SignatorySetError::Malformed)?
        .0 as usize;
    let needed = count
        .checked_mul(DEFINITION_BYTES)
        .and_then(|definitions| definitions.checked_add(DEPOSIT_BYTES))
        .ok_or(SignatorySetError::Malformed)?;
    if input.len() != needed {
        return Err(SignatorySetError::Malformed);
    }
    Ok(input
        .chunks_exact(DEFINITION_BYTES)
        .take(count)
        .filter_map(|definition| {
            definition
                .get(..32)
                .and_then(|delegate| <[u8; 32]>::try_from(delegate).ok())
        })
        .map(AccountId)
        .collect())
}
