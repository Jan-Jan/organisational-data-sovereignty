//! The invitation exchange (REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-yazum3):
//! the Invite a Member sends and the Invite reply the invitee returns, as
//! Blobs (Base64 of postcard), parsed at this edge, naming the field that
//! fails (LLR-b7wgpf). Nothing here is verified: an Invite's sender and its
//! Organisation name are whatever the sender chose (HAZ-qfb95k, RC-wzb48r).

use std::collections::BTreeSet;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use org_node::service::{Joiner, OrgService};
use org_node::{DevicePublicKey, Handle, Name, OrgId, OrgNodeError, PersonPublicKey, PersonaId, Surname};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::submit::{submit_commit_send, ChainWriter};

/// The identifier an Invite carries and its reply echoes (REQ-65xqp8): 32
/// bytes this device drew at random. Not secret. The app's own type: it binds
/// a reply to the Invite this device issued and never reaches org-node
/// (REQ-tcutr6 as amended).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InviteId([u8; 32]);

impl InviteId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// An Invite as parsed: what the inviter typed, the Organisation it names,
/// the inviter's devices, and the identifier its reply must echo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    pub org_name: String,
    pub org_id: OrgId,
    pub invitee_name: String,
    pub inviter_device_keys: Vec<DevicePublicKey>,
    pub invite_id: InviteId,
}

/// An Invite reply as parsed: the Invite it answers and the Persona the
/// invitee chose to join with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteReply {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
}

#[derive(Serialize, Deserialize)]
struct WireInvite {
    org_name: String,
    org_id: Vec<u8>,
    invitee_name: String,
    inviter_device_keys: Vec<Vec<u8>>,
    invite_id: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WireReply {
    org_id: Vec<u8>,
    invite_id: Vec<u8>,
    member_key: Vec<u8>,
    device_key: Vec<u8>,
    handle: String,
    name: String,
    surname: String,
}

fn exact<const N: usize>(field: &str, bytes: &[u8]) -> Result<[u8; N], String> {
    bytes.try_into().map_err(|_| format!("{field}: expected {N} bytes, got {}", bytes.len()))
}

fn named<T, E: std::fmt::Display>(field: &str, r: Result<T, E>) -> Result<T, String> {
    r.map_err(|e| format!("{field}: {e}"))
}

fn unarmour<T: for<'de> Deserialize<'de>>(what: &str, blob: &str) -> Result<T, String> {
    let bytes = STANDARD.decode(blob.trim()).map_err(|e| format!("{what}: not a Blob: {e}"))?;
    postcard::from_bytes(&bytes).map_err(|e| format!("{what}: does not decode: {e}"))
}

fn armour<T: Serialize>(value: &T) -> Result<String, String> {
    postcard::to_allocvec(value).map(|b| STANDARD.encode(b)).map_err(|e| format!("encode: {e}"))
}

impl Invite {
    /// Parse an Invite Blob, naming the field that fails (LLR-b7wgpf).
    pub fn parse(blob: &str) -> Result<Self, String> {
        let w: WireInvite = unarmour("invite", blob)?;
        Ok(Self {
            org_name: w.org_name,
            org_id: OrgId::new(exact("invite.org_id", &w.org_id)?),
            invitee_name: w.invitee_name,
            inviter_device_keys: w
                .inviter_device_keys
                .iter()
                .map(|k| named("invite.inviter_device_keys", DevicePublicKey::parse(&exact("invite.inviter_device_keys", k)?)))
                .collect::<Result<_, _>>()?,
            invite_id: InviteId::new(exact("invite.invite_id", &w.invite_id)?),
        })
    }

    pub fn encode(&self) -> Result<String, String> {
        armour(&WireInvite {
            org_name: self.org_name.clone(),
            org_id: self.org_id.as_bytes().to_vec(),
            invitee_name: self.invitee_name.clone(),
            inviter_device_keys: self.inviter_device_keys.iter().map(|k| k.as_bytes().to_vec()).collect(),
            invite_id: self.invite_id.as_bytes().to_vec(),
        })
    }

    /// A Blob from raw parts, so a test can present one this software never
    /// writes.
    #[cfg(feature = "test-support")]
    pub fn wire_for_test(org_name: &str, org_id: &[u8], invitee: &str, keys: &[[u8; 32]], invite_id: &[u8]) -> String {
        armour(&WireInvite {
            org_name: org_name.into(),
            org_id: org_id.to_vec(),
            invitee_name: invitee.into(),
            inviter_device_keys: keys.iter().map(|k| k.to_vec()).collect(),
            invite_id: invite_id.to_vec(),
        })
        .unwrap_or_default()
    }
}

impl InviteReply {
    /// Parse an Invite reply Blob, naming the field that fails (LLR-b7wgpf).
    pub fn parse(blob: &str) -> Result<Self, String> {
        let w: WireReply = unarmour("reply", blob)?;
        Ok(Self {
            org_id: OrgId::new(exact("reply.org_id", &w.org_id)?),
            invite_id: InviteId::new(exact("reply.invite_id", &w.invite_id)?),
            member_key: named("reply.member_key", PersonPublicKey::parse(&exact("reply.member_key", &w.member_key)?))?,
            device_key: named("reply.device_key", DevicePublicKey::parse(&exact("reply.device_key", &w.device_key)?))?,
            handle: named("reply.handle", Handle::parse(&w.handle))?,
            name: named("reply.name", Name::parse(&w.name))?,
            surname: named("reply.surname", Surname::parse(&w.surname))?,
        })
    }

    pub fn encode(&self) -> Result<String, String> {
        armour(&WireReply {
            org_id: self.org_id.as_bytes().to_vec(),
            invite_id: self.invite_id.as_bytes().to_vec(),
            member_key: self.member_key.as_bytes().to_vec(),
            device_key: self.device_key.as_bytes().to_vec(),
            handle: self.handle.to_string(),
            name: self.name.to_string(),
            surname: self.surname.to_string(),
        })
    }

    /// A Blob from raw parts, so a test can present one this software never
    /// writes.
    #[cfg(feature = "test-support")]
    pub fn wire_for_test(
        org_id: &[u8],
        invite_id: &[u8],
        member_key: &[u8; 32],
        device_key: &[u8; 32],
        handle: &str,
        name: &str,
        surname: &str,
    ) -> String {
        armour(&WireReply {
            org_id: org_id.to_vec(),
            invite_id: invite_id.to_vec(),
            member_key: member_key.to_vec(),
            device_key: device_key.to_vec(),
            handle: handle.into(),
            name: name.into(),
            surname: surname.into(),
        })
        .unwrap_or_default()
    }
}

/// One outstanding Invite as the file holds it: the Organisation it was
/// issued for and its identifier, each as hex.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilePair {
    org_id: String,
    invite_id: String,
}

/// The Invites this device issued and has not yet seen acted on, each as the
/// (Organisation, invite identifier) pair it was issued for, kept in a file
/// (LLR-f35pda).
pub struct OutstandingInvites {
    path: PathBuf,
    pairs: BTreeSet<(OrgId, InviteId)>,
}

impl OutstandingInvites {
    /// Open the file at `path`; a missing file holds none, and a file that is
    /// not a JSON list of `{org_id, invite_id}` hex pairs — the earlier list
    /// of bare identifiers among them — is refused, naming it.
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let refused = |e: &dyn std::fmt::Display| format!("outstanding invites {}: {e}", path.display());
        let pairs = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str::<Vec<FilePair>>(&text)
                .map_err(|e| refused(&e))?
                .iter()
                .map(|p| {
                    let org = hex::decode(&p.org_id).map_err(|e| refused(&format!("org_id: {e}")))?;
                    let id = hex::decode(&p.invite_id).map_err(|e| refused(&format!("invite_id: {e}")))?;
                    Ok((
                        OrgId::new(exact("org_id", &org).map_err(|e| refused(&e))?),
                        InviteId::new(exact("invite_id", &id).map_err(|e| refused(&e))?),
                    ))
                })
                .collect::<Result<_, String>>()?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
            Err(e) => return Err(refused(&e)),
        };
        Ok(Self { path, pairs })
    }

    fn save(&self) -> Result<(), String> {
        let file: Vec<FilePair> = self
            .pairs
            .iter()
            .map(|(org, id)| FilePair { org_id: hex::encode(org.as_bytes()), invite_id: hex::encode(id.as_bytes()) })
            .collect();
        let text = serde_json::to_string(&file).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, text).map_err(|e| format!("outstanding invites {}: {e}", self.path.display()))
    }

    /// Draw a fresh identifier for `org_id`, keep the pair, and save before
    /// returning.
    pub fn issue<R: RngCore + CryptoRng>(&mut self, rng: &mut R, org_id: OrgId) -> Result<InviteId, String> {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        let id = InviteId::new(bytes);
        self.pairs.insert((org_id, id));
        self.save()?;
        Ok(id)
    }

    /// Whether `id` is outstanding for `org_id` — for that Organisation only.
    pub fn holds(&self, org_id: OrgId, id: &InviteId) -> bool {
        self.pairs.contains(&(org_id, *id))
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Drop the pair (`org_id`, `id`), keeping the others, and save before
    /// returning.
    pub fn settle(&mut self, org_id: OrgId, id: &InviteId) -> Result<(), String> {
        self.pairs.remove(&(org_id, *id));
        self.save()
    }
}

/// LLR-9sraks: an Invite for `org_id`, carrying the inviter's DevicePublicKeys
/// (every Persona of this device bound to it) and a fresh outstanding id.
pub fn issue_invite<R: RngCore + CryptoRng>(
    svc: &OrgService,
    outstanding: &mut OutstandingInvites,
    rng: &mut R,
    org_id: OrgId,
    org_name: &str,
    invitee_name: &str,
) -> Result<String, String> {
    let keys: Vec<DevicePublicKey> = svc
        .list_personas()
        .iter()
        .filter(|p| p.org_id == Some(org_id))
        .map(|p| svc.persona_public_keys(&p.persona_id).map(|(_, d)| d).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    if keys.is_empty() {
        return Err("no Persona of this device belongs to that Organisation".into());
    }
    let invite_id = outstanding.issue(rng, org_id)?;
    Invite { org_name: org_name.into(), org_id, invitee_name: invitee_name.into(), inviter_device_keys: keys, invite_id }
        .encode()
}

/// LLR-w4mhd4: the reply to `invite_blob` from `persona_id`, only once the
/// user confirmed and only for a Persona bound to no Organisation
/// (LLR-rt8gdz); declares the expected admission to the Invite's
/// Organisation (REQ-tcutr6 as amended).
pub fn produce_reply<R: RngCore + CryptoRng>(
    svc: &mut OrgService,
    rng: &mut R,
    invite_blob: &str,
    persona_id: &PersonaId,
    confirmed: bool,
) -> Result<String, String> {
    if !confirmed {
        return Err("confirm first: nothing has verified who sent this Invite or the name it states, and the reply reveals the chosen Persona's handle, name and surname".into());
    }
    let invite = Invite::parse(invite_blob)?;
    let p = svc
        .list_personas()
        .iter()
        .find(|p| &p.persona_id == persona_id)
        .cloned()
        .ok_or("no such Persona")?;
    // LLR-rt8gdz: one Persona, one Organisation.
    if p.org_id.is_some() {
        return Err(OrgNodeError::PersonaAlreadyBound { persona_id: persona_id.clone() }.to_string());
    }
    let (member_key, device_key) = svc.persona_public_keys(persona_id).map_err(|e| e.to_string())?;
    let reply = InviteReply {
        org_id: invite.org_id,
        invite_id: invite.invite_id,
        member_key,
        device_key,
        handle: p.handle,
        name: p.name,
        surname: p.surname,
    }
    .encode()?;
    svc.expect_admission(rng, invite.org_id).map_err(|e| e.to_string())?;
    Ok(reply)
}

/// LLR-gha5f6: a reply whose (Organisation, invite id) pair is outstanding,
/// else refused — an id outstanding for another Organisation is not.
pub fn check_reply(outstanding: &OutstandingInvites, reply_blob: &str) -> Result<InviteReply, String> {
    let reply = InviteReply::parse(reply_blob)?;
    if !outstanding.holds(reply.org_id, &reply.invite_id) {
        return Err("this reply names no Invite this device has outstanding".into());
    }
    Ok(reply)
}

/// LLR-gha5f6, LLR-qhjp6g: admit the person a reply names, through the
/// chain, to the Organisation its outstanding pair names, sent to the
/// reply's Device alone (LLR-q225ws); the Invite is settled once the admission
/// has committed, even when the send that follows fails. `org_id` is the
/// operator's selection: it is not trusted, and must be that Organisation.
pub async fn admit_reply<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    outstanding: &mut OutstandingInvites,
    rng: &mut R,
    org_id: OrgId,
    reply_blob: &str,
    peer_addr: Option<iroh::EndpointAddr>,
) -> Result<org_node::MemberId, String> {
    let reply = check_reply(outstanding, reply_blob)?;
    if reply.org_id != org_id {
        return Err("this reply is for another Organisation".into());
    }
    let joiner = Joiner {
        handle: reply.handle.clone(),
        name: reply.name.clone(),
        surname: reply.surname.clone(),
        member_key: reply.member_key,
        device_key: reply.device_key,
    };
    let update = svc.admit_member(rng, org_id, &joiner).map_err(|e| e.to_string())?;
    let sent = submit_commit_send(svc, writer, rng, &update, reply.device_key, peer_addr).await;
    let admitted = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == org_id)
        .and_then(|o| o.trie_members.iter().find(|m| m.member_key == reply.member_key).map(|m| m.id));
    match admitted {
        Some(id) => {
            outstanding.settle(org_id, &reply.invite_id)?;
            sent?;
            Ok(id)
        }
        None => sent.and(Err("the admission did not commit".into())),
    }
}
