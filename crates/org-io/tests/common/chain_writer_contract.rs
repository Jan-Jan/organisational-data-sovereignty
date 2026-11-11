//! The contract of `test_support::FakeChain`'s `ChainWriter` (TEST_GUIDELINES Doubles 3): the
//! assertions a founding's two steps must meet,
//! run through the handle against whatever chain `io` was built over. `tests/chain_write.rs`
//! runs them over `FakeChain`, in the gate; `tests/chain_writer_contract.rs` runs them over the
//! production `OnChainWriter` on a chopsticks fork, outside every gate.
//!
//! 1. The founding's write is `Included`, naming the founding's epoch.
//! 2. The Organisation recorded with the proxy is the H160 on-chain-client's `h160_of` gives
//!    for the proxy account (LLR-5tpdxw): the slot the chain keys the genesis under.
//! 3. A second write of the same founding calls `found_at` again at the stored proxy; the slot
//!    holds a genesis already, so the contract refuses the record, and the report names the
//!    `RecordGenesis` step as `write.rs` maps on-chain-client's `WriteError::Step`
//!    (LLR-sq5pbq).
//! 4. The verification of the founding adopts it, under the Organisation of step 2.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use org_io::{HeldTarget, OrgIo, Verified, WriteReport};
use org_node::OrgId;

/// Run the founding contract over `io`, which holds no founding yet.
pub async fn founding_contract(io: &OrgIo) {
    let proposed = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let target = HeldTarget::Founding(proposed.founder);

    let written = io.write_organisation_to_chain(target).await.unwrap();
    assert_eq!(written, WriteReport::Included { epoch: proposed.epoch }, "1: the founding is included");

    // The Organisation recorded with the proxy, as the handle lists the founder's Persona.
    let org_id = io
        .personas()
        .await
        .into_iter()
        .find(|persona| persona.device_key == proposed.founder)
        .and_then(|persona| persona.org_id)
        .expect("the Organisation recorded");

    // The stored proxy is read from the refusal of the second write, which names the proxy
    // `found_at` was given again (the stored one), not from org-node's store through the handle.
    let again = io.write_organisation_to_chain(target).await.unwrap();
    let WriteReport::Refused { failure } = again else { panic!("3: a second genesis at the proxy is refused: {again:?}") };
    let reason = failure.to_string();
    assert!(reason.contains("genesis step RecordGenesis did not execute"), "3: the refusal names the RecordGenesis step: {reason}");
    let proxy: [u8; 32] = reason
        .split(|character: char| !character.is_ascii_hexdigit())
        .find(|word| word.len() == 64)
        .and_then(|word| hex::decode(word).ok())
        .and_then(|bytes| bytes.try_into().ok())
        .unwrap_or_else(|| panic!("3: and the stored proxy: {reason}"));
    assert_eq!(org_id, OrgId::new(on_chain_client::h160_of(proxy)), "2: the Organisation is h160_of(the stored proxy)");

    let verified = io.verify_organisation(target).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "4: the founding is adopted: {verified:?}");
    assert_eq!(io.organisations().await.iter().map(|org| org.org_id).collect::<Vec<_>>(), vec![org_id], "4: under h160_of(proxy)");
}
