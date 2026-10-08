//! The own-admin check (SDD-f4khqn): whether this node's user is a signatory
//! of the multisig that controls an Organisation's pure proxy. The chain can
//! confirm a signatory set but cannot list one (pallet-multisig stores no
//! member list), so the check is of this node's own status only, from its
//! own account and its configured co-signers (REQ-f3eu9n).

use async_trait::async_trait;
use on_chain_client::write::multisig::multi_account_id;
use on_chain_client::write::AccountId;
use on_chain_client::{OrgAdmin, OrgRegistryClient};
use org_node::OrgId;

/// Whether this node is an admin of an Organisation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminStatus {
    Admin,
    NotAdmin,
}

/// Why the own-admin check gave no answer. A failed read is never
/// `NotAdmin` (LLR-c9fyun).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnAdminError {
    /// This node holds no signatory key.
    NotConfigured,
    /// A signatory-set read failed.
    Read(String),
}

impl std::fmt::Display for OwnAdminError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Built without `dev-seed`, says the seed is read only by
            // development builds (REQ-8zuka3).
            Self::NotConfigured => write!(formatter, "this node holds no signatory key; {}", crate::submit::NOT_CONFIGURED),
            Self::Read(reason) => write!(formatter, "the signatory-set read failed: {reason}"),
        }
    }
}

impl std::error::Error for OwnAdminError {}

/// The account every org-io dispatch is made from (LLR-qhyc3n): `own` alone
/// when no co-signer other than `own` is configured, otherwise the
/// threshold-1 multisig of `own` and the co-signers, de-duplicated (and
/// sorted by `multi_account_id`).
pub fn controller_of(own: AccountId, co_signers: &[AccountId]) -> AccountId {
    let mut signatories = vec![own];
    for co_signer in co_signers {
        if !signatories.contains(co_signer) {
            signatories.push(*co_signer);
        }
    }
    if signatories.len() == 1 {
        own
    } else {
        multi_account_id(&signatories, 1)
    }
}

/// `Admin` exactly when the H160 maps to an account and that account's
/// delegates include `controller` (LLR-vyd5d3).
pub fn admin_status(controller: AccountId, original_account: Option<AccountId>, delegates: &[AccountId]) -> AdminStatus {
    if original_account.is_some() && delegates.contains(&controller) {
        AdminStatus::Admin
    } else {
        AdminStatus::NotAdmin
    }
}

/// The two signatory-set reads (on-chain-client's REQ-8p2veg, REQ-v8jczx).
#[async_trait]
pub trait SignatorySetReader: Send + Sync {
    /// The account the Organisation's H160 maps to, if any.
    async fn original_account(&self, org_id: OrgId) -> Result<Option<AccountId>, String>;
    /// `account`'s proxy delegates.
    async fn proxy_delegates(&self, account: AccountId) -> Result<Vec<AccountId>, String>;
}

/// The check over a reader (LLR-c9fyun): the delegates are read only when
/// the H160 maps to an account, and a failed read is an error.
pub(crate) async fn own_admin_status(
    reader: &dyn SignatorySetReader,
    own: Option<AccountId>,
    co_signers: &[AccountId],
    org_id: OrgId,
) -> Result<AdminStatus, OwnAdminError> {
    let own = own.ok_or(OwnAdminError::NotConfigured)?;
    let original_account = reader.original_account(org_id).await.map_err(OwnAdminError::Read)?;
    let delegates = match original_account {
        Some(account) => reader.proxy_delegates(account).await.map_err(OwnAdminError::Read)?,
        None => Vec::new(),
    };
    Ok(admin_status(controller_of(own, co_signers), original_account, &delegates))
}

/// The production reads (SDD-z85ux9's shell), over on-chain-client's
/// `OrgRegistryClient` at the latest finalised block.
pub struct OnChainSignatorySetReader {
    registry: OrgRegistryClient,
}

impl OnChainSignatorySetReader {
    pub fn new(registry: OrgRegistryClient) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl SignatorySetReader for OnChainSignatorySetReader {
    async fn original_account(&self, org_id: OrgId) -> Result<Option<AccountId>, String> {
        self.registry.original_account(OrgAdmin(*org_id.as_bytes())).await.map_err(|error| error.to_string())
    }

    async fn proxy_delegates(&self, account: AccountId) -> Result<Vec<AccountId>, String> {
        self.registry.proxy_delegates(account).await.map_err(|error| error.to_string())
    }
}

/// The reads when the chain is not configured: every read refused, with the
/// same message as the unconfigured write and read (REQ-8zuka3).
pub struct SignatorySetNotConfigured;

#[async_trait]
impl SignatorySetReader for SignatorySetNotConfigured {
    async fn original_account(&self, _org_id: OrgId) -> Result<Option<AccountId>, String> {
        Err(crate::submit::NOT_CONFIGURED.into())
    }

    async fn proxy_delegates(&self, _account: AccountId) -> Result<Vec<AccountId>, String> {
        Err(crate::submit::NOT_CONFIGURED.into())
    }
}
