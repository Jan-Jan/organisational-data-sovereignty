//! The own-admin check (SDD-f4khqn): whether this node's user is a signatory
//! of the multisig that controls an Organisation's pure proxy. The chain can
//! confirm a signatory set but cannot list one (pallet-multisig stores no
//! member list), so the check is of this node's own status only, from its
//! own account and its configured co-signers (REQ-ntkjk6).

use async_trait::async_trait;
use on_chain_client::write::multisig::multi_account_id;
pub use on_chain_client::write::signatory_set::ProxyDefinition;
use on_chain_client::write::AccountId;
use on_chain_client::{OrgAdmin, OrgRegistryClient};
use org_node::OrgId;

/// An Organisation's signatory set as one read returns it (on-chain-client's
/// `SignatorySet`): the mapped account, its proxy definitions in stored
/// order, and the runtime's `Any` index, all at one finalised block. Public
/// chain data only.
pub type SignatorySetView = on_chain_client::write::signatory_set::SignatorySet;

/// Whether this node is an admin of an Organisation (LLR-2hkqv2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminStatus {
    /// A definition `(controller, Any, 0)` exists: the chain dispatches this
    /// node's writes at once.
    Admin,
    /// The controller is a delegate only with another proxy type or a
    /// non-zero delay: the chain would refuse its writes.
    DelegateCannotDispatch,
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

/// `Admin` exactly when the H160 maps to an account and some definition is
/// `(controller, Any, 0)`, with `Any` the index the read resolved from the
/// runtime's metadata; `DelegateCannotDispatch` when the account is mapped
/// and definitions name `controller`, but none that way; `NotAdmin`
/// otherwise (LLR-2hkqv2).
pub fn admin_status(controller: AccountId, set: &SignatorySetView) -> AdminStatus {
    if set.original_account.is_none() {
        return AdminStatus::NotAdmin;
    }
    let naming_controller: Vec<&ProxyDefinition> =
        set.definitions.iter().filter(|definition| definition.delegate == controller).collect();
    if naming_controller.iter().any(|definition| definition.proxy_type == set.any_proxy_type && definition.delay == 0) {
        AdminStatus::Admin
    } else if naming_controller.is_empty() {
        AdminStatus::NotAdmin
    } else {
        AdminStatus::DelegateCannotDispatch
    }
}

/// The one signatory-set read, at one finalised block (on-chain-client's
/// REQ-b8x2py).
#[async_trait]
pub trait SignatorySetReader: Send + Sync {
    /// The account the Organisation's H160 maps to, that account's proxy
    /// definitions, and the runtime's `Any` index, all at one block.
    async fn signatory_set(&self, org_id: OrgId) -> Result<SignatorySetView, String>;
}

/// The check over a reader (LLR-c9fyun, LLR-2hkqv2): one signatory-set read,
/// and a failed read is an error.
pub(crate) async fn own_admin_status(
    reader: &dyn SignatorySetReader,
    own: Option<AccountId>,
    co_signers: &[AccountId],
    org_id: OrgId,
) -> Result<AdminStatus, OwnAdminError> {
    let own = own.ok_or(OwnAdminError::NotConfigured)?;
    let set = reader.signatory_set(org_id).await.map_err(OwnAdminError::Read)?;
    Ok(admin_status(controller_of(own, co_signers), &set))
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
    async fn signatory_set(&self, org_id: OrgId) -> Result<SignatorySetView, String> {
        self.registry.signatory_set(OrgAdmin(*org_id.as_bytes())).await.map_err(|error| error.to_string())
    }
}

/// The read when the chain is not configured: refused, with the same message
/// as the unconfigured write and read (REQ-8zuka3).
pub struct SignatorySetNotConfigured;

#[async_trait]
impl SignatorySetReader for SignatorySetNotConfigured {
    async fn signatory_set(&self, _org_id: OrgId) -> Result<SignatorySetView, String> {
        Err(crate::submit::NOT_CONFIGURED.into())
    }
}
