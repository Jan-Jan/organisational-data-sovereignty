//! The contract of `test_support::FakeChain`'s other halves (TEST_GUIDELINES Doubles 3;
//! LLR-js7wgk, LLR-j4nkx5, LLR-bwerd2): its state and finalised-block reads (`StateReader`), its write
//! window and update (`ChainWriter`) and its signatory-set read (`SignatorySetReader`), at those
//! seams, against whatever chain `ChainUnderTest` holds. `tests/chain_write.rs` runs them over
//! `FakeChain`, in the gate; `tests/chain_writer_contract.rs` runs them over the production halves
//! (`test_support::production_chain`) on a chopsticks fork, outside every gate, and skips when no
//! fork can be had. The founding's two steps have their own contract, through the handle
//! (`chain_writer_contract.rs`).
//!
//! 1. The finalised block is read; a write window read after it is born at or after it, and
//!    a finalised-block read after the window at or after its birth; once the chain finalises
//!    another block, the read is later.
//! 2. An Organisation the chain holds no state for reads as `Ok(None)`, and its signatory set
//!    as no mapped account and no definition.
//! 3. A founding written through the writer reads back as its root and key at epoch 1.
//! 4. Once its account is mapped and its proxy definitions are the controller's founding one,
//!    the signatory-set read answers the mapped account and that one definition,
//!    `(controller, Any, 0)`, with the runtime's `Any` index.
//! 5. An update at the read state's epoch, mortal from a window just read, moves the state one
//!    epoch on to its root and key.
//! 6. An update from an epoch the chain has left is refused as the call made through the proxy
//!    failed (on-chain-client's `WriteError::InnerCallFailed`, the contract's refusal of a stale
//!    expected epoch), and the state is unchanged.
//!
//! What the fake's other switches answer is pinned apart, in the fake's own run
//! (`fake_chain_meets_the_reader_and_writer_contract`): `state_hidden` as clause 2, and
//! `epoch_ahead` as the state clause 5 gives for an update at the same root and key, and
//! `parse_refusing` as the real reader's parse refuses a key org-node refuses
//! (`chain_read::org_state_from_chain`, which `OnChainStateReader::read_state` calls; no writer
//! org-io holds can put such a key on a real chain, since an `OrgPublicKey` is parsed).
//! `read_failing`, the window's and the writes' failures and hangs, are fault injectors (Doubles 5).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::future::Future;

use on_chain_client::write::{AccountId, WriteError};
use org_io::chain_read::StateReader;
use org_io::signatory::{ProxyDefinition, SignatorySetReader};
use org_io::write::{ChainWriteError, ChainWriter};
use org_node::{ChainAccount, Epoch, OrgId, OrgState, RootHash};

use super::journey::stranger_key;

/// The chain the contract runs against, at org-io's three seams.
pub struct ChainUnderTest<'a> {
    pub reader: &'a dyn StateReader,
    pub writer: &'a dyn ChainWriter,
    pub signatory_set: &'a dyn SignatorySetReader,
}

/// An Organisation identifier no founding on either chain gives (`h160_of` of a proxy account).
pub fn never_founded() -> OrgId {
    OrgId::new([0x99; 20])
}

/// Clauses 1, 2, 3, 5 and 6. `finalise` makes the chain finalise at least one more block.
/// Returns the founded Organisation and its proxy, for clause 4.
pub async fn reader_and_writer_contract<F, Fut>(chain: &ChainUnderTest<'_>, mut finalise: F) -> (OrgId, ChainAccount)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let before = chain.reader.finalised_block().await.expect("1: the finalised block is read");
    let window = chain.writer.write_window().await.expect("1: the write window is read");
    let after = chain.reader.finalised_block().await.unwrap();
    assert!(before <= window.birth_number && window.birth_number <= after, "1: born at the finalised block: {before} {window:?} {after}");
    finalise().await;
    let later = chain.reader.finalised_block().await.unwrap();
    assert!(later > after, "1: a block finalised since is read: {after} then {later}");

    assert_eq!(chain.reader.read_state(never_founded()).await.unwrap(), None, "2: no state");
    let set = chain.signatory_set.signatory_set(never_founded()).await.unwrap();
    assert_eq!((set.original_account, set.definitions.len()), (None, 0), "2: no mapped account, no definition: {set:?}");

    let (org_id, proxy) = chain.writer.create_proxy().await.expect("3: the proxy is created");
    let (root, founding_key) = (RootHash::new([0x31; 32]), stranger_key(0x31));
    chain.writer.found_at(proxy, root, founding_key).await.expect("3: the genesis is recorded");
    finalise().await;
    let founded = chain.reader.read_state(org_id).await.unwrap();
    assert_eq!(founded, Some(OrgState { root_hash: root, org_pub_key: founding_key, epoch: Epoch::new(1) }), "3: the genesis reads back");

    let (next_root, next_key) = (RootHash::new([0x32; 32]), stranger_key(0x32));
    let window = chain.writer.write_window().await.unwrap();
    chain.writer.update(proxy, next_root, next_key, Epoch::new(1), window).await.expect("5: the update executes");
    finalise().await;
    let next = OrgState { root_hash: next_root, org_pub_key: next_key, epoch: Epoch::new(2) };
    assert_eq!(chain.reader.read_state(org_id).await.unwrap(), Some(next), "5: one epoch on, its root and key");

    let window = chain.writer.write_window().await.unwrap();
    let stale = chain.writer.update(proxy, RootHash::new([0x33; 32]), stranger_key(0x33), Epoch::new(1), window).await;
    assert!(matches!(stale, Err(ChainWriteError::Write(WriteError::InnerCallFailed(_)))), "6: a stale expected epoch is refused: {stale:?}");
    finalise().await;
    assert_eq!(chain.reader.read_state(org_id).await.unwrap(), Some(next), "6: the state is unchanged");
    (org_id, proxy)
}

/// Clause 4, for `org_id` founded at `proxy` and `controller`'s founding definition.
pub async fn signatory_set_contract(chain: &ChainUnderTest<'_>, org_id: OrgId, proxy: ChainAccount, controller: AccountId) {
    let set = chain.signatory_set.signatory_set(org_id).await.expect("4: the signatory set is read");
    assert_eq!(set.original_account, Some(AccountId(*proxy.as_bytes())), "4: the mapped account is the proxy");
    assert_eq!(
        set.definitions,
        vec![ProxyDefinition { delegate: controller, proxy_type: set.any_proxy_type, delay: 0 }],
        "4: the controller's founding definition"
    );
}
