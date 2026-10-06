//! Fixtures shared by the app's test targets. Not a test target: each target
//! that uses it declares `mod support;`.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use ods_poc_lib::submit::ChainWriter;
use org_node::service::MockChainOps;
use org_node::{ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgId, OrgPublicKey, RootHash};

/// A substitute chain write over org-node's `MockChainOps`, which the service
/// reads. `failing` makes every write fail; `hanging` makes every write never
/// finish.
pub struct FakeWriter {
    chain: MockChainOps,
    pub failing: AtomicBool,
    pub hanging: AtomicBool,
    proxies: Mutex<HashMap<[u8; 32], OrgId>>,
}

impl FakeWriter {
    pub fn over(chain: &MockChainOps) -> Self {
        Self {
            chain: chain.clone(),
            failing: AtomicBool::new(false),
            hanging: AtomicBool::new(false),
            proxies: Mutex::new(HashMap::new()),
        }
    }

    /// A writer whose every call never finishes.
    pub fn never(chain: &MockChainOps) -> Self {
        let writer = Self::over(chain);
        writer.hanging.store(true, Ordering::SeqCst);
        writer
    }

    /// How many Organisations this writer has written a genesis for.
    pub fn geneses(&self) -> usize {
        self.proxies.lock().unwrap().len()
    }

    async fn gate(&self) -> Result<(), String> {
        if self.hanging.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        if self.failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ChainWriter for FakeWriter {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        self.gate().await?;
        let org = self.chain.apply_genesis(root, key);
        let mut proxy = [0u8; 32];
        proxy[..20].copy_from_slice(org.as_bytes());
        self.proxies.lock().unwrap().insert(proxy, org);
        Ok((org, ChainAccount::new(proxy)))
    }

    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, epoch: Epoch) -> Result<(), String> {
        self.gate().await?;
        let org = *self.proxies.lock().unwrap().get(proxy.as_bytes()).ok_or("unknown proxy")?;
        self.chain.apply_update(org, root, key, epoch).map_err(|e| e.to_string())
    }
}

/// A member key and a device key that parse, as an Invite reply carries them.
pub fn reply_keys() -> ([u8; 32], [u8; 32]) {
    let member = MemberSeed::from([8; 32]).x25519_keypair().member_key().expect("member key");
    let device = DeviceSeed::from([7; 32]).signing_keypair().device_key().expect("device key");
    (*member.as_bytes(), *device.as_bytes())
}
