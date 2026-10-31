#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The chain read hands org-node a value: present, absent, or an error —
//! never absence for a failure (LLR-rm9x4z).

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_io::chain_read::{org_state_from_chain, read_org_state};
use org_node::{Epoch, OrgId, OrgNodeError, OrgPrivateKey, RootHash};

fn x25519_key(seed: u8) -> [u8; 32] {
    *OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap().as_bytes()
}

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState { root_hash: OnChainRootHash([0x33; 32]), org_pub_key: OrgPubKey(key), epoch: ChainEpoch(7) }
}

struct ScriptedReader(Result<Option<ChainOrgState>, String>);

#[async_trait::async_trait]
impl org_io::chain_read::RawStateSource for ScriptedReader {
    async fn fetch(&self, _org_id: OrgId) -> Result<Option<ChainOrgState>, String> {
        self.0.clone()
    }
}

// verifies: LLR-rm9x4z
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = x25519_key(0x11);
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
}

// verifies: LLR-rm9x4z
#[test]
fn a_chain_state_whose_key_org_node_refuses_is_an_error() {
    assert_eq!(org_state_from_chain(chain_state([0u8; 32])).unwrap_err(), OrgNodeError::InvalidOrgPublicKey);
}

// verifies: LLR-rm9x4z
#[tokio::test]
async fn present_absent_failed_and_refused_reads_are_four_different_answers() {
    let org = OrgId::new([1u8; 20]);
    let present = read_org_state(&ScriptedReader(Ok(Some(chain_state(x25519_key(0x11))))), org).await.unwrap();
    assert!(present.is_some());
    assert_eq!(read_org_state(&ScriptedReader(Ok(None)), org).await.unwrap(), None);
    let failed = read_org_state(&ScriptedReader(Err("registry read failed".into())), org).await.unwrap_err();
    assert_eq!(failed, OrgNodeError::Chain("get_org_state: registry read failed".into()));
    let refused = read_org_state(&ScriptedReader(Ok(Some(chain_state([0u8; 32])))), org).await.unwrap_err();
    assert_eq!(refused, OrgNodeError::InvalidOrgPublicKey);
}

/// Every `.rs` file under `org-io/src`, relative path and code lines (line
/// comments dropped).
fn org_io_code() -> Vec<(String, Vec<String>)> {
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<(String, Vec<String>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let relative = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                let text = std::fs::read_to_string(&path).unwrap();
                let code = text.lines().map(|line| line.split("//").next().unwrap_or("").to_string()).collect();
                out.push((relative, code));
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![];
    walk(&root, &root, &mut files);
    files
}

// The chain's state is read only by verification (and the operator's
// preflight): `read_state` and `finalised_block` are called only in
// `verify.rs` and the preflight (`preflight.rs`, `bin/preflight.rs`; the
// files that define them, `chain_read.rs`, `connect.rs` and
// `test_support.rs`, call neither, and no file is exempt from the scan), and
// the receive, send, write, proposal, invitation, Person, Persona,
// acknowledgement and retention files name no `StateReader` at all.
// verifies: LLR-k3zmqb
#[test]
fn the_state_reader_is_named_only_in_verify_and_preflight() {
    let callers = ["verify.rs", "preflight.rs", "bin/preflight.rs"];
    let silent = ["receiver.rs", "send.rs", "write.rs", "propose.rs", "exchange.rs", "person.rs", "persona.rs", "acknowledge.rs", "retention.rs"];
    let mut found = vec![];
    let mut verify_calls = 0;
    for (file, code) in org_io_code() {
        for (index, line) in code.iter().enumerate() {
            let calls = line.contains(".read_state(") || line.contains(".finalised_block(");
            if calls && file == "verify.rs" {
                verify_calls += 1;
            }
            if calls && !callers.contains(&file.as_str()) {
                found.push(format!("{file}:{}: reads the chain: {}", index + 1, line.trim()));
            }
            if line.contains("StateReader") && silent.contains(&file.as_str()) {
                found.push(format!("{file}:{}: names the state reader: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(verify_calls >= 2, "the scan finds verification's two reads ({verify_calls})");
    assert!(found.is_empty(), "the chain read outside verification:\n{}", found.join("\n"));
}
