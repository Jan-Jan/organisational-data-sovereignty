//! The signatory-set reads' pure half (SDD-rxfu6h): decoding the stored
//! values of `Revive.OriginalAccount` and `Proxy.Proxies`, the signatory set
//! they make up (`read_signatory_set`, over any `SignatorySetSource`), and
//! the index of the runtime's `Any` proxy type in its metadata
//! (`any_proxy_type_index`). The source that fetches the bytes at one pinned
//! block is in `client.rs`.

use core::future::Future;

use parity_scale_codec::{Compact, Decode};

use super::AccountId;
use crate::types::OrgAdmin;

/// A stored value that is not what the runtime stores there, or a runtime
/// whose proxy type has no `Any` variant (LLR-sc79gq).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatorySetError {
    Malformed,
    NoAnyProxyType,
}

impl core::fmt::Display for SignatorySetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SignatorySetError::Malformed => f.write_str("malformed signatory-set storage value"),
            SignatorySetError::NoAnyProxyType => f.write_str("the runtime's proxy type has no `Any` variant"),
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

/// One stored proxy definition: the delegate, the runtime's proxy-type
/// variant index and the announcement delay in blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProxyDefinition {
    pub delegate: AccountId,
    pub proxy_type: u8,
    pub delay: u32,
}

/// An Organisation's signatory set, read at one finalised block
/// (LLR-sc79gq): the account its H160 maps to, that account's proxy
/// definitions in stored order (empty when unmapped), and the index of the
/// runtime's `Any` proxy type at that block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatorySet {
    pub original_account: Option<AccountId>,
    pub definitions: Vec<ProxyDefinition>,
    pub any_proxy_type: u8,
}

/// `Proxy.Proxies`' value: the definitions with their type and delay, in
/// stored order (LLR-n2vx5q). The length prefix is checked against the bytes
/// that follow before anything is allocated.
pub fn decode_proxy_definitions(bytes: &[u8]) -> Result<Vec<ProxyDefinition>, SignatorySetError> {
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
    input
        .chunks_exact(DEFINITION_BYTES)
        .take(count)
        .map(decode_definition)
        .collect()
}

/// One `DEFINITION_BYTES` chunk: 32-byte delegate, type byte, LE delay.
fn decode_definition(chunk: &[u8]) -> Result<ProxyDefinition, SignatorySetError> {
    let (delegate, rest) = chunk.split_first_chunk::<32>().ok_or(SignatorySetError::Malformed)?;
    let (proxy_type, delay) = rest.split_first().ok_or(SignatorySetError::Malformed)?;
    let delay = <[u8; 4]>::try_from(delay).map_err(|_| SignatorySetError::Malformed)?;
    Ok(ProxyDefinition {
        delegate: AccountId(*delegate),
        proxy_type: *proxy_type,
        delay: u32::from_le_bytes(delay),
    })
}

/// A storage value a signatory set is read from (LLR-sc79gq).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatorySetEntry {
    /// `Revive.OriginalAccount`, keyed by the Organisation's H160.
    OriginalAccount(OrgAdmin),
    /// `Proxy.Proxies`, keyed by the account the H160 maps to.
    Proxies(AccountId),
}

impl SignatorySetEntry {
    /// The pallet, the storage entry and the key's bytes.
    pub fn storage(&self) -> (&'static str, &'static str, &[u8]) {
        match self {
            SignatorySetEntry::OriginalAccount(admin) => ("Revive", "OriginalAccount", admin.0.as_slice()),
            SignatorySetEntry::Proxies(account) => ("Proxy", "Proxies", account.0.as_slice()),
        }
    }
}

/// Where `read_signatory_set` reads its values from: one chain state.
/// `client.rs` implements it over one pinned block (SDD-3b8zef's shell); a
/// gated test substitutes it.
pub trait SignatorySetSource {
    type Error;
    /// The raw value stored at `entry`, or `None` when none is stored.
    fn fetch(&self, entry: SignatorySetEntry) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send;
    /// A value that did not decode, as this source's error.
    fn decode_error(error: SignatorySetError) -> Self::Error;
}

/// An Organisation's signatory set from `source` (LLR-sc79gq): the account
/// `admin` maps to, that account's proxy definitions (read only when it is
/// mapped; none when none are stored) and `any_proxy_type`. A failed read is
/// the read's error and a value that does not decode is a decode error —
/// never "not mapped" or "no definitions".
pub async fn read_signatory_set<S: SignatorySetSource + Sync>(
    source: &S,
    admin: OrgAdmin,
    any_proxy_type: u8,
) -> Result<SignatorySet, S::Error> {
    let original_account = match source.fetch(SignatorySetEntry::OriginalAccount(admin)).await? {
        None => None,
        Some(bytes) => Some(decode_original_account(&bytes).map_err(S::decode_error)?),
    };
    let definitions = match original_account {
        None => Vec::new(),
        Some(account) => match source.fetch(SignatorySetEntry::Proxies(account)).await? {
            None => Vec::new(),
            Some(bytes) => decode_proxy_definitions(&bytes).map_err(S::decode_error)?,
        },
    };
    Ok(SignatorySet {
        original_account,
        definitions,
        any_proxy_type,
    })
}

/// The index of the variant named `Any` of the runtime's proxy type in
/// `metadata` (LLR-sc79gq): the type of `Proxy.create_pure`'s `proxy_type`
/// argument, with `Any` encoded against it by name — the index comes from the
/// metadata, never from a constant. A runtime with no such argument, or whose
/// proxy type has no `Any` that encodes as one byte, is `NoAnyProxyType`.
pub fn any_proxy_type_index(metadata: &subxt::Metadata) -> Result<u8, SignatorySetError> {
    use subxt::dynamic::Value;
    use subxt::ext::scale_encode::EncodeAsType;
    use subxt::ext::scale_value::Composite;

    let proxy_type_id = metadata
        .pallet_by_name("Proxy")
        .and_then(|pallet| pallet.call_variant_by_name("create_pure"))
        .and_then(|call| {
            call.fields
                .iter()
                .find(|field| field.name.as_deref() == Some("proxy_type"))
        })
        .map(|field| field.ty.id)
        .ok_or(SignatorySetError::NoAnyProxyType)?;
    let any = Value::variant("Any", Composite::unnamed(vec![]));
    let encoded = any
        .encode_as_type(proxy_type_id, metadata.types())
        .map_err(|_| SignatorySetError::NoAnyProxyType)?;
    match encoded.as_slice() {
        [index] => Ok(*index),
        _ => Err(SignatorySetError::NoAnyProxyType),
    }
}
