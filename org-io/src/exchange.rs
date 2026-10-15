//! The invitation exchange runner (SDD-rr7dtw): org-node
//! builds and parses the Invite and its reply and keeps the outstanding
//! pairs and the expectations in each Persona's sealed store; org-io runs
//! the exchange, writing each store change before it returns
//! (LLR-h9qf3s, LLR-9cb443, LLR-y4dg74; REQ-prjja8, REQ-tcutr6,
//! REQ-65xqp8).
//!
//! The Blobs these return carry Device and member public keys, names and
//! the invite identifier, by design: an Invite and its reply are public.

use org_node::invitation::{Invite, InviteReply};
use org_node::service::Joiner;
use org_node::{MemberId, OrgId, OrgNodeError, PersonaId};
use rand_core::{CryptoRng, RngCore};

use crate::peer_addr::PeerAddr;
use crate::send::SendReport;
use crate::transport::TransportError;
use crate::{OperationError, OrgIo, StoreFailure};

/// What the user must have confirmed before a reply is produced (the app's
/// confirmation requirement): the refusal names it.
const CONFIRM_FIRST: &str = "confirm first: nothing has verified who sent this Invite or the name it states, \
     and the reply reveals the chosen Persona's handle, name and surname";

/// Why an exchange operation returned no Blob or admitted no one: public
/// data only — org-node's typed refusals, the store's failure, the
/// submission's report and a send report.
#[derive(Debug)]
pub enum ExchangeError {
    /// `produce_reply` without the user's confirmation: nothing parsed,
    /// built or written (LLR-9cb443).
    NotConfirmed,
    /// org-node refused: a Blob that does not parse (naming the field), a
    /// reply not outstanding or for another Organisation, or a build it
    /// refuses (an unknown or bound Persona, an Organisation no Persona is
    /// bound to).
    Node(OrgNodeError),
    /// The operation's store write failed: no Blob is returned and the
    /// Persona was reloaded from its file, or dropped (LLR-murjr6).
    Store(StoreFailure),
    /// The admission's submission failed (the submission's own report): a
    /// failure before the chain write submitted nothing.
    Submit(String),
    /// The admission committed and its sends are reported, but settling the
    /// reply's invite identifier failed; the pair may still be outstanding.
    Settle { admitted: AdmitReport, error: OperationError },
    /// `produce_reply` while the node receives: the replying Persona's
    /// endpoint could not be bound, so the admission the reply invites would
    /// not be received; nothing built or written (PR-y6t5kt, LLR-z35vbh).
    Bind(TransportError),
}

impl From<OrgNodeError> for ExchangeError {
    fn from(error: OrgNodeError) -> Self {
        Self::Node(error)
    }
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfirmed => write!(formatter, "{CONFIRM_FIRST}"),
            Self::Node(error) => write!(formatter, "{error}"),
            Self::Store(failure) => write!(formatter, "nothing returned: {failure}"),
            Self::Submit(report) => write!(formatter, "{report}"),
            Self::Settle { error, .. } => write!(formatter, "the admission committed, but settling the Invite failed: {error}"),
            Self::Bind(error) => write!(formatter, "no reply: this Persona's endpoint could not be bound to receive on: {error}"),
        }
    }
}

impl std::error::Error for ExchangeError {}

/// What `admit_reply` reports of an admission that committed: the new
/// Member's id and what became of each recipient's body (LLR-y4dg74).
#[derive(Debug)]
pub struct AdmitReport {
    pub member_id: MemberId,
    pub sends: SendReport,
}

/// The joiner a reply names.
fn joiner_of(reply: &InviteReply) -> Joiner {
    Joiner {
        handle: reply.handle.clone(),
        name: reply.name.clone(),
        surname: reply.surname.clone(),
        member_key: reply.member_key,
        device_key: reply.device_key,
    }
}

impl OrgIo {
    /// An Invite for `org_id` from the Persona bound to it (org-node's
    /// `build_invite`), its identifier kept as outstanding in that Persona's
    /// store, which is written before the Blob is returned; a failed write
    /// returns the failure and no Blob (LLR-h9qf3s, LLR-9sraks).
    pub async fn issue_invite<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        org_id: OrgId,
        org_name: &str,
        invitee_name: &str,
    ) -> Result<String, ExchangeError> {
        let mut service = self.service.lock().await;
        let (invite, write) = service.build_invite(rng, org_id, org_name, invitee_name)?;
        self.local.write(&mut service, write).map_err(ExchangeError::Store)?;
        Ok(invite.encode()?)
    }

    /// The reply to `invite_blob` from `persona_id`, only once the user
    /// confirmed: unconfirmed, it parses nothing and writes nothing
    /// (`NotConfirmed`). Otherwise org-node parses the Invite and builds the
    /// reply, recording the expected admission in the Persona's store, which
    /// is written before the Blob is returned (LLR-9cb443, LLR-w4mhd4,
    /// LLR-rt8gdz).
    ///
    /// When the node receives, the Persona's endpoint is first bound and
    /// accepts (PR-y6t5kt, LLR-z35vbh); a bind
    /// that fails is `ExchangeError::Bind`, with nothing built or written.
    pub async fn produce_reply<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        invite_blob: &str,
        persona_id: &PersonaId,
        confirmed: bool,
    ) -> Result<String, ExchangeError> {
        if !confirmed {
            return Err(ExchangeError::NotConfirmed);
        }
        let invite = Invite::parse(invite_blob)?;
        let mut service = self.service.lock().await;
        let (_, device_key) = service.persona_public_keys(persona_id)?;
        self.local.endpoints.accept_if_receiving(&device_key).await.map_err(ExchangeError::Bind)?;
        let (reply, write) = service.build_reply(rng, &invite, persona_id)?;
        self.local.write(&mut service, write).map_err(ExchangeError::Store)?;
        Ok(reply.encode()?)
    }

    /// The reply `reply_blob`, parsed, when its (Organisation, invite
    /// identifier) pair is outstanding here (org-node's `check_reply`, for
    /// the Organisation the reply names); writes nothing (LLR-gha5f6).
    pub async fn import_reply(&self, reply_blob: &str) -> Result<InviteReply, ExchangeError> {
        let reply = InviteReply::parse(reply_blob)?;
        self.service.lock().await.check_reply(&reply, reply.org_id)?;
        Ok(reply)
    }

    /// Admit the person `reply_blob` names to `org_id`, the operator's
    /// selection (LLR-y4dg74, LLR-gha5f6). Stopping at the first refusal:
    /// parse the reply; `check_reply` for `org_id`; org-node's
    /// `admit_member`, its store written; the submission (chain write, read
    /// back, `commit_update`, its store written, the send to every
    /// recipient); `settle`, its store written. Once the admission has
    /// committed it settles whatever the sends reported; a refusal before
    /// the chain write submits nothing, and a failed submission leaves the
    /// pair outstanding unless the admission committed.
    pub async fn admit_reply<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        org_id: OrgId,
        reply_blob: &str,
        peer_addr: Option<PeerAddr>,
    ) -> Result<AdmitReport, ExchangeError> {
        let reply = InviteReply::parse(reply_blob)?;
        self.service.lock().await.check_reply(&reply, org_id)?;
        let update = self.admit_member(rng, org_id, &joiner_of(&reply)).await.map_err(|error| match error {
            OperationError::Node(refused) => ExchangeError::Node(refused),
            OperationError::Store(failure) => ExchangeError::Store(failure),
        })?;
        // A failed submission is reported as it says: nothing committed, or
        // committed but not stored (the Persona then reloaded from its file,
        // which holds no commit). Either way the file holds no admission, so
        // the pair stays outstanding.
        let submitted = self.submit_commit_send(rng, &update, peer_addr).await.map_err(ExchangeError::Submit)?;
        // A committed, stored admission puts its Member in the record, so
        // this refusal guards a state the public API does not reach, and no
        // test drives it: reaching it needs project code faked. It keeps the
        // per-Device send failures of the update that did commit.
        let member_id = self
            .organisation(org_id)
            .await
            .and_then(|record| record.members.iter().find(|member| member.member_key == reply.member_key).map(|member| member.id))
            .ok_or_else(|| {
                let mut report = String::from("the admission committed, but its Member is not in the record");
                if let Some(failed) = submitted.sends.failure_summary() {
                    report.push_str(&format!("; sending it failed: {failed}"));
                }
                ExchangeError::Submit(report)
            })?;
        let admitted = AdmitReport { member_id, sends: submitted.sends };
        let mut service = self.service.lock().await;
        let settled = service
            .settle(rng, reply.org_id, reply.invite_id)
            .map_err(OperationError::Node)
            .and_then(|write| self.local.write(&mut service, write).map_err(OperationError::Store));
        match settled {
            Ok(()) => Ok(admitted),
            Err(error) => Err(ExchangeError::Settle { admitted, error }),
        }
    }
}
