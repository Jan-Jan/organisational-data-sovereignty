#![cfg(feature = "write")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The pure decoders behind the signatory-set reads (SDD-rxfu6h), and the
//! chain-free half of the pinned read (LLR-sc79gq): the `Any` lookup over a
//! runtime's metadata, here a minimal one built from derived `TypeInfo`, and
//! the assembly of the set from the values one block gives.

use std::collections::BTreeMap;
use std::sync::Mutex;

use on_chain_client::OrgAdmin;
use on_chain_client::write::AccountId;
use on_chain_client::write::signatory_set::{
    ProxyDefinition, SignatorySet, SignatorySetEntry, SignatorySetError, SignatorySetSource,
    any_proxy_type_index, decode_original_account, decode_proxy_definitions,
    decode_proxy_delegates, read_signatory_set,
};
use parity_scale_codec::{Compact, Encode};
use scale_info::{MetaType, PortableRegistry, Registry, TypeInfo};
use subxt::ext::frame_metadata::v16;

const CLIENT_SOURCE: &str = include_str!("../src/client.rs");

/// The runtime types the metadata tests describe. Only their type
/// information is used; the values are never built.
#[allow(dead_code, non_camel_case_types)]
mod runtime {
    use scale_info::TypeInfo;

    /// Paseo Asset Hub's shape: `Any` first.
    #[derive(TypeInfo)]
    pub enum ProxyType {
        Any,
        NonTransfer,
        CancelProxy,
    }

    /// `Any` sorted elsewhere, at an index no constant would guess.
    #[derive(TypeInfo)]
    pub enum ProxyTypeAnyAtFive {
        NonTransfer,
        CancelProxy,
        #[codec(index = 5)]
        Any,
    }

    /// No variant named `Any`.
    #[derive(TypeInfo)]
    pub enum ProxyTypeWithoutAny {
        NonTransfer,
        CancelProxy,
    }

    /// A variant named `Any` that carries a field.
    #[derive(TypeInfo)]
    pub enum ProxyTypeAnyWithField {
        Any(u32),
        NonTransfer,
    }

    /// `Proxy`'s calls, with `create_pure`'s `proxy_type` of type `P`.
    #[derive(TypeInfo)]
    pub enum ProxyCall<P: TypeInfo + 'static> {
        create_pure { proxy_type: P, delay: u32, index: u16 },
        remove_proxies,
    }

    /// `create_pure` without a `proxy_type` argument.
    #[derive(TypeInfo)]
    pub enum ProxyCallWithoutProxyType {
        create_pure { delay: u32, index: u16 },
    }

    /// No `create_pure` at all.
    #[derive(TypeInfo)]
    pub enum ProxyCallWithoutCreatePure {
        remove_proxies,
    }
}

/// A runtime's metadata with one pallet, `pallet`, whose calls are of type
/// `calls` (none when `None`).
fn metadata(pallet: &str, calls: Option<MetaType>) -> subxt::Metadata {
    let mut registry = Registry::new();
    let unit = registry.register_type(&MetaType::new::<()>());
    let calls = calls.map(|ty| v16::PalletCallMetadata {
        ty: registry.register_type(&ty),
        deprecation_info: v16::EnumDeprecationInfo(BTreeMap::new()),
    });
    let pallet = v16::PalletMetadata {
        name: pallet.to_string(),
        storage: None,
        calls,
        event: None,
        constants: vec![],
        error: None,
        associated_types: vec![],
        view_functions: vec![],
        index: 42,
        docs: vec![],
        deprecation_info: v16::ItemDeprecationInfo::NotDeprecated,
    };
    let types: PortableRegistry = registry.into();
    subxt::Metadata::try_from(v16::RuntimeMetadataV16 {
        types,
        pallets: vec![pallet],
        extrinsic: v16::ExtrinsicMetadata {
            versions: vec![4],
            address_ty: unit,
            call_ty: unit,
            signature_ty: unit,
            transaction_extensions_by_version: BTreeMap::new(),
            transaction_extensions: vec![],
        },
        apis: vec![],
        outer_enums: v16::OuterEnums { call_enum_ty: unit, event_enum_ty: unit, error_enum_ty: unit },
        custom: v16::CustomMetadata { map: BTreeMap::new() },
    })
    .unwrap()
}

/// `Proxy`'s metadata with `create_pure`'s `proxy_type` of type `P`.
fn proxy_metadata<P: TypeInfo + 'static>() -> subxt::Metadata {
    metadata("Proxy", Some(MetaType::new::<runtime::ProxyCall<P>>()))
}

// Normal: the index is the variant named `Any` of `create_pure`'s
// `proxy_type`, read from the metadata — 0 where it sorts first, 5 where the
// runtime gives it index 5.
// verifies: LLR-sc79gq
#[test]
fn the_any_index_is_read_from_the_runtimes_metadata_by_name() {
    assert_eq!(any_proxy_type_index(&proxy_metadata::<runtime::ProxyType>()), Ok(0));
    assert_eq!(any_proxy_type_index(&proxy_metadata::<runtime::ProxyTypeAnyAtFive>()), Ok(5));
}

// Abnormal: a runtime with no `Proxy` pallet, a `Proxy` pallet with no calls
// or no `create_pure`, a `create_pure` with no `proxy_type`, a proxy type
// with no `Any`, an `Any` that carries a field, and a proxy type that is not
// an enum are each `NoAnyProxyType` — never a guessed index.
// verifies: LLR-sc79gq
#[test]
fn a_runtime_without_an_any_proxy_type_is_refused() {
    let refused = [
        ("no Proxy pallet", metadata("Utility", Some(MetaType::new::<runtime::ProxyCall<runtime::ProxyType>>()))),
        ("no calls", metadata("Proxy", None)),
        ("no create_pure", metadata("Proxy", Some(MetaType::new::<runtime::ProxyCallWithoutCreatePure>()))),
        ("no proxy_type", metadata("Proxy", Some(MetaType::new::<runtime::ProxyCallWithoutProxyType>()))),
        ("no Any", proxy_metadata::<runtime::ProxyTypeWithoutAny>()),
        ("Any with a field", proxy_metadata::<runtime::ProxyTypeAnyWithField>()),
        ("not an enum", proxy_metadata::<u8>()),
    ];
    for (case, metadata) in refused {
        assert_eq!(any_proxy_type_index(&metadata), Err(SignatorySetError::NoAnyProxyType), "{case}");
    }
}

/// What a read of the signatory set ended in, as the fake source reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReadError {
    Read(&'static str),
    Decode(SignatorySetError),
}

/// One block's two values, as a fake chain answers them; records each entry
/// asked for, in order.
struct OneBlock {
    original_account: Result<Option<Vec<u8>>, &'static str>,
    proxies: Result<Option<Vec<u8>>, &'static str>,
    asked: Mutex<Vec<SignatorySetEntry>>,
}

impl OneBlock {
    fn new(
        original_account: Result<Option<Vec<u8>>, &'static str>,
        proxies: Result<Option<Vec<u8>>, &'static str>,
    ) -> Self {
        Self { original_account, proxies, asked: Mutex::new(Vec::new()) }
    }
    fn asked(&self) -> Vec<SignatorySetEntry> {
        self.asked.lock().unwrap().clone()
    }
}

impl SignatorySetSource for OneBlock {
    type Error = ReadError;
    async fn fetch(&self, entry: SignatorySetEntry) -> Result<Option<Vec<u8>>, ReadError> {
        self.asked.lock().unwrap().push(entry);
        let answer = match entry {
            SignatorySetEntry::OriginalAccount(_) => &self.original_account,
            SignatorySetEntry::Proxies(_) => &self.proxies,
        };
        answer.clone().map_err(ReadError::Read)
    }
    fn decode_error(error: SignatorySetError) -> ReadError {
        ReadError::Decode(error)
    }
}

const ADMIN: OrgAdmin = OrgAdmin([0xad; 20]);
const MAPPED: AccountId = AccountId([0x5a; 32]);

// Normal: a mapped H160's account, then that account's definitions, are read
// from the one source, and the `Any` index given is the set's; an unmapped
// H160 reads no proxies, and a mapped account with none stored has none.
// verifies: LLR-sc79gq
#[tokio::test]
async fn the_set_is_the_account_its_definitions_and_the_any_index() {
    let proxies = typed_definitions(&[([1u8; 32], 0, 0), ([2u8; 32], 3, 7)]);
    let block = OneBlock::new(Ok(Some(MAPPED.0.to_vec())), Ok(Some(proxies)));
    assert_eq!(
        read_signatory_set(&block, ADMIN, 5).await,
        Ok(SignatorySet {
            original_account: Some(MAPPED),
            definitions: vec![
                ProxyDefinition { delegate: AccountId([1u8; 32]), proxy_type: 0, delay: 0 },
                ProxyDefinition { delegate: AccountId([2u8; 32]), proxy_type: 3, delay: 7 },
            ],
            any_proxy_type: 5,
        })
    );
    assert_eq!(
        block.asked(),
        vec![SignatorySetEntry::OriginalAccount(ADMIN), SignatorySetEntry::Proxies(MAPPED)]
    );

    let unmapped = OneBlock::new(Ok(None), Err("never read"));
    assert_eq!(
        read_signatory_set(&unmapped, ADMIN, 0).await,
        Ok(SignatorySet { original_account: None, definitions: vec![], any_proxy_type: 0 })
    );
    assert_eq!(unmapped.asked(), vec![SignatorySetEntry::OriginalAccount(ADMIN)], "no proxies read");

    let no_proxies = OneBlock::new(Ok(Some(MAPPED.0.to_vec())), Ok(None));
    assert_eq!(
        read_signatory_set(&no_proxies, ADMIN, 0).await,
        Ok(SignatorySet { original_account: Some(MAPPED), definitions: vec![], any_proxy_type: 0 })
    );
}

// Abnormal: a failed read of either value is that read's error, and a value
// that does not decode is a decode error — never "not mapped" or "no
// delegates"; nothing is read after the first failure.
// verifies: LLR-sc79gq
#[tokio::test]
async fn a_failed_read_or_an_undecodable_value_is_an_error() {
    let good_proxies = typed_definitions(&[([1u8; 32], 0, 0)]);
    let cases = [
        (OneBlock::new(Err("node unreachable"), Ok(Some(good_proxies.clone()))), ReadError::Read("node unreachable"), 1),
        (OneBlock::new(Ok(Some(vec![0x5a; 31])), Ok(Some(good_proxies))), ReadError::Decode(SignatorySetError::Malformed), 1),
        (OneBlock::new(Ok(Some(MAPPED.0.to_vec())), Err("proxies unreadable")), ReadError::Read("proxies unreadable"), 2),
        (OneBlock::new(Ok(Some(MAPPED.0.to_vec())), Ok(Some(vec![0u8; 3]))), ReadError::Decode(SignatorySetError::Malformed), 2),
    ];
    for (block, expected, reads) in cases {
        assert_eq!(read_signatory_set(&block, ADMIN, 0).await, Err(expected.clone()), "{expected:?}");
        assert_eq!(block.asked().len(), reads, "{expected:?}: reads made");
    }
}

// The storage each entry names: `Revive.OriginalAccount` keyed by the H160's
// twenty bytes, `Proxy.Proxies` keyed by the account's thirty-two.
// verifies: LLR-sc79gq
#[test]
fn each_entry_names_its_storage_and_key() {
    assert_eq!(
        SignatorySetEntry::OriginalAccount(ADMIN).storage(),
        ("Revive", "OriginalAccount", ADMIN.0.as_slice())
    );
    assert_eq!(SignatorySetEntry::Proxies(MAPPED).storage(), ("Proxy", "Proxies", MAPPED.0.as_slice()));
}

/// The SCALE value `Proxy.Proxies` stores for these definitions.
fn typed_definitions(definitions: &[([u8; 32], u8, u32)]) -> Vec<u8> {
    let mut bytes = Compact(definitions.len() as u32).encode();
    for (delegate, proxy_type, delay) in definitions {
        bytes.extend_from_slice(delegate);
        bytes.push(*proxy_type);
        bytes.extend_from_slice(&delay.to_le_bytes());
    }
    bytes.extend_from_slice(&1_000u128.to_le_bytes()); // deposit
    bytes
}

// Normal: every definition keeps its delegate, type byte and delay, in the
// order the chain stored them; no definitions is an empty list.
// verifies: LLR-n2vx5q, LLR-a9bb7b
#[test]
fn definitions_decode_with_type_and_delay_in_stored_order() {
    let bytes = typed_definitions(&[([1u8; 32], 0, 0), ([1u8; 32], 3, 0), ([2u8; 32], 0, 10)]);
    assert_eq!(
        decode_proxy_definitions(&bytes).unwrap(),
        vec![
            ProxyDefinition { delegate: AccountId([1u8; 32]), proxy_type: 0, delay: 0 },
            ProxyDefinition { delegate: AccountId([1u8; 32]), proxy_type: 3, delay: 0 },
            ProxyDefinition { delegate: AccountId([2u8; 32]), proxy_type: 0, delay: 10 },
        ]
    );
    let largest = typed_definitions(&[([9u8; 32], u8::MAX, u32::MAX)]);
    assert_eq!(
        decode_proxy_definitions(&largest).unwrap(),
        vec![ProxyDefinition { delegate: AccountId([9u8; 32]), proxy_type: u8::MAX, delay: u32::MAX }]
    );
    assert_eq!(decode_proxy_definitions(&typed_definitions(&[])).unwrap(), vec![]);
}

// Abnormal: trailing bytes, too few bytes, a length prefix larger than the
// bytes that follow, and no bytes at all are each `Malformed`.
// verifies: LLR-n2vx5q, LLR-a9bb7b
#[test]
fn a_malformed_definitions_value_is_refused() {
    let good = typed_definitions(&[([1u8; 32], 0, 0), ([2u8; 32], 1, 5)]);
    let mut trailing = good.clone();
    trailing.push(0);
    assert_eq!(decode_proxy_definitions(&trailing), Err(SignatorySetError::Malformed));
    assert_eq!(decode_proxy_definitions(&good[..good.len() - 1]), Err(SignatorySetError::Malformed));
    let mut no_deposit = Compact(1u32).encode();
    no_deposit.extend_from_slice(&[0u8; 37]);
    assert_eq!(decode_proxy_definitions(&no_deposit), Err(SignatorySetError::Malformed));
    let mut oversized = Compact(u32::MAX).encode();
    oversized.extend_from_slice(&[0u8; 16]);
    assert_eq!(decode_proxy_definitions(&oversized), Err(SignatorySetError::Malformed));
    assert_eq!(decode_proxy_definitions(&[]), Err(SignatorySetError::Malformed));
}

/// The function in `source` from `signature` to its closing brace, written
/// as `closing` (the brace at the function's own indentation).
fn function_body<'a>(source: &'a str, signature: &str, closing: &str) -> &'a str {
    let start = source.find(signature).unwrap_or_else(|| panic!("no `{signature}` in client.rs"));
    let rest = &source[start..];
    let end = rest.find(closing).unwrap_or_else(|| panic!("no end of `{signature}`"));
    &rest[..end]
}

// The pinned read (PR-ksm2ua's reproduction): `signatory_set` resolves the
// latest finalised block once and every read in it is made at that block —
// it calls neither of the two reads that each resolve their own block.
// Before: the two values were read by two calls, each at its own block.
// *Amended 2026-10-13 (task gate1fix; unmerged, LLR-sc79gq minted in this
// change):* the reads and the `Any` lookup moved into chain-free functions
// (`read_signatory_set` over a `SignatorySetSource`, `any_proxy_type_index`
// over metadata), tested above; this scan now pins that `signatory_set`
// hands both the one block — its metadata, and a source whose every fetch
// is at it.
// verifies: LLR-sc79gq, PR-ksm2ua
#[test]
fn the_signatory_set_is_read_at_one_block() {
    let body = function_body(CLIENT_SOURCE, "pub async fn signatory_set(", "\n    }\n");
    assert_eq!(body.matches("at_current_block()").count(), 1, "the block is resolved once:\n{body}");
    assert!(!body.contains("at_block("), "no second block is resolved:\n{body}");
    for own_block_read in
        ["self.original_account(", "self.proxy_delegates(", "self.fetch_storage_raw(", ".storage()"]
    {
        assert!(!body.contains(own_block_read), "`{own_block_read}` reads at its own block:\n{body}");
    }
    assert!(
        body.contains("any_proxy_type_index(at_block.metadata_ref())"),
        "the `Any` lookup takes the pinned block's metadata:\n{body}"
    );
    assert!(
        body.contains("read_signatory_set(&PinnedBlock(&at_block)"),
        "the values are read from the pinned block:\n{body}"
    );
    assert_eq!(body.matches("at_block").count(), 3, "the pinned block is used for nothing else:\n{body}");
    let source = function_body(CLIENT_SOURCE, "SignatorySetSource for PinnedBlock", "\n}\n");
    assert_eq!(
        source.matches("fetch_storage_raw_at(").count(),
        1,
        "every fetch goes through the one fetch:\n{source}"
    );
    assert!(source.contains("fetch_storage_raw_at(self.0,"), "at the pinned block:\n{source}");
    assert!(!source.contains("at_current_block") && !source.contains("at_block("), "no other block:\n{source}");
    let fetch = function_body(CLIENT_SOURCE, "async fn fetch_storage_raw_at(", "\n}\n");
    assert!(!fetch.contains("at_current_block"), "the fetch resolves no block of its own:\n{fetch}");
}

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
