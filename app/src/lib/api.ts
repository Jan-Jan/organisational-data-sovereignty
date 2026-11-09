/**
 * Typed wrappers around Tauri invoke/listen for the ODS commands + the events the receiver and the verification announce.
 *
 * Command signatures mirror commands.rs exactly (camelCase arg names per Tauri convention).
 * Return shapes mirror the Rust DTOs of commands.rs and carry their names (PersonaDto, OrgDto, InviteDto, ConnectionStatus, ...).
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ---------------------------------------------------------------------------
// DTOs (mirror Rust structs)
// ---------------------------------------------------------------------------

export interface PersonaDto {
	persona_id: string;
	org_id: string | null;
	/** null for a Persona that holds no Handle yet, such as a provisional Member (LLR-pguhw5). */
	handle: string | null;
	name: string;
	surname: string;
	/** "Proposed" | "Active" | "Revoked" (Debug format of enum) */
	status: string;
}

/** LLR-g8wfy9: the last verification of an Organisation on this handle. */
export type VerificationDto =
	| { status: 'in_step'; epoch: number }
	| { status: 'adopted'; epoch: number }
	| { status: 'removed' }
	| { status: 'behind'; chain_epoch: number }
	| { status: 'inconsistent'; reason: string }
	| { status: 'unverified'; reason: string }
	| { status: 'not_checked' };

export interface OrgDto {
	org_id: string;    // 40 hex chars
	epoch: number;
	root_hash: string; // 64 hex chars
	member_count: number;
	verification: VerificationDto;
}

/** LLR-b7ywy4: an Invite's public fields; nothing is stored. */
export interface InviteDto {
	org_name: string;
	org_id: string; // 40 hex chars
	invitee_name: string;
	issuer_device_key: string; // 64 hex chars
}

/** LLR-rg9vc7: the new Persona a Member proposal created. */
export interface ProposedPersonaDto {
	persona_id: string;
	status: string;
}

/** LLR-8cz828: org-io's outcome of an admission request, tagged by `result`. */
export type AdmissionDto =
	| { result: 'delivered' }
	| { result: 'already_received' }
	| { result: 'invite_used_by_another_device' }
	| { result: 'refused' }
	| { result: 'not_delivered'; reason: string };

/** LLR-rywe8f, LLR-nkg8sb: one Device's outcome of a send or a request. */
export type DeviceReportDto =
	| { device: string; result: 'sent' }
	| { device: string; result: 'failed'; reason: string };

/**
 * LLR-n64wvr: a provisional change an adopted epoch left out. org-io reports
 * the request's name and surname, and a revoked Member's Handle, when it
 * still holds them; otherwise the field is `null` (names table OPEN-1).
 */
export type DiscardedDto =
	| { kind: 'admission'; invite_id: string; name: string | null; surname: string | null; handle: string }
	| { kind: 'revocation'; member_id: string; handle: string | null };

/**
 * LLR-jf63e4: which target an entry is for. An
 * Organisation's entry has its `org_id` and no `founder`. Every provisional
 * founding's entry carries `founder`, its founder's Device key (64 hex),
 * beside `org_id` `null` before its proxy is recorded and the proxy's
 * Organisation identifier after.
 */
export type EntryOrg = { org_id: string; founder?: undefined } | { org_id: string | null; founder: string };

/** LLR-jf63e4: what a verification settled of an unknown chain write (org-io's `Verified::Settled`). */
export type WriteLabelDto = 'adopted' | 'not_executed' | 'deprecated' | 'still_unknown';

/** LLR-jf63e4: one verification result, tagged by `result`. */
export type VerificationEntryDto = EntryOrg & { write?: WriteLabelDto } & (
		| { result: 'in_step'; epoch: number }
		| {
				result: 'adopted';
				epoch: number;
				own: boolean;
				discarded: DiscardedDto[];
				sends?: DeviceReportDto[];
				sends_refused?: string;
		  }
		| { result: 'removed' }
		| { result: 'behind'; chain_epoch: number; requests?: DeviceReportDto[]; requests_refused?: string }
		| { result: 'inconsistent'; reason: string }
		| { result: 'unverified'; reason: string }
	);

/**
 * Which of the two ways this installation reaches other nodes.
 * Mirrors the Rust `TransportModeName`, which serialises lowercase.
 */
export type TransportMode = 'networked' | 'loopback';

export interface ConnectionStatus {
	/** True only when a real chain client was constructed at startup. */
	chain_configured: boolean;
	/**
	 * REQ-e4ah9h / REQ-bvx4nh: the endpoint the running configuration was built
	 * from, or null. Never re-read from the environment by the backend.
	 */
	chain_ws: string | null;
	contract_h160: string | null;
	/** REQ-645jq9. */
	transport_mode: TransportMode;
	data_dir: string;
}

// ---------------------------------------------------------------------------
// Event payloads
// ---------------------------------------------------------------------------

export interface MembershipUpdatedPayload {
	org_id: string;
	epoch: number;
	root: string;
}

/** LLR-ygr6gd: `org_id` is null for a provisional founding whose proxy is not recorded. */
export interface RevokedPayload {
	org_id: string | null;
}

/**
 * epoch-changed payload.
 *
 * REQ-tw4cb5: this is NO LONGER emitted on self-delete. A revoked persona has
 * deleted its organisation record, so there is no epoch to report; the previous
 * backend emitted a literal 0, which is genesis and therefore a reachable,
 * meaningful value (HAZ-5ha5vv).
 */
export interface EpochChangedPayload {
	org_id: string;
	epoch: number;
}

export interface ReceiverErrorPayload {
	message: string;
}

/** REQ-affyf5: `org_id` is null when the failure happened before an
 * organisation was identified. */
export interface VerificationFailedPayload {
	org_id: string | null;
	message: string;
}

/** REQ-2k7ys4 / REQ-dp95pv: an update verified but its record could not be
 * read back. No membership, epoch or verification event accompanies it. */
export interface RecordUnreadablePayload {
	/** LLR-acb5g4: null for a provisional founding whose proxy is not recorded. */
	org_id: string | null;
}

/** REQ-jfxah3: the receiver loop has exited and is no longer running. */
export interface ReceiverStoppedPayload {
	reason: string;
}

/** LLR-y7nc6x: one kept admission request as list_admission_requests returns it; also the admission-requested payload, field for field (LLR-dp45mm). */
export interface AdmissionRequestDto {
	org_id: string; // 40 hex chars
	invite_id: string; // 64 hex chars
	invitee_name: string;
	name: string;
	surname: string;
	preferred_handles: string[];
	device_public_key: string; // 64 hex chars
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * LLR-gmb4p8: record the admission of the request kept for `inviteId`, with
 * the Admin's chosen Handle (blank: org-io chooses), once confirmed
 * (LLR-cr9nmw). These four values alone: no Organisation secret, reply or
 * peer address.
 */
export function proposeAdmission(
	orgId: string,
	inviteId: string,
	handle: string | null,
	confirmed: boolean
): Promise<void> {
	return invoke<void>('propose_admission', { orgId, inviteId, handle, confirmed });
}

/** List all local personas (no key material returned). */
export function listPersonas(): Promise<PersonaDto[]> {
	return invoke<PersonaDto[]>('list_personas');
}

/** List all local org records. */
export function listOrgs(): Promise<OrgDto[]> {
	return invoke<OrgDto[]>('list_orgs');
}

/** LLR-5mw2dj: the admission requests kept before the start; no argument. */
export function listAdmissionRequests(): Promise<AdmissionRequestDto[]> {
	return invoke<AdmissionRequestDto[]>('list_admission_requests');
}

/** LLR-dmfvv6: one Device an Organisation's store holds as revoked, as list_removed_devices returns it. */
export interface RemovedDeviceDto {
	member_id: string; // 64 hex chars
	device_key: string; // 64 hex chars
	acknowledged_epoch: number | null;
}

/** LLR-dmfvv6, LLR-x9qbhu: the Devices `orgId`'s store holds as revoked, in org-io's order. */
export function listRemovedDevices(orgId: string): Promise<RemovedDeviceDto[]> {
	return invoke<RemovedDeviceDto[]>('list_removed_devices', { orgId });
}

/** LLR-y3zykm: an Invite for this Organisation; no Organisation name and no address are passed. */
export function issueInvite(orgId: string, inviteeName: string): Promise<string> {
	return invoke<string>('issue_invite', { orgId, inviteeName });
}

/** LLR-b7ywy4: read an Invite Blob; a field that does not parse is named. */
export function readInvite(blob: string): Promise<InviteDto> {
	return invoke<InviteDto>('read_invite', { blob });
}

/** LLR-rg9vc7: a new Persona proposed as a Member from an Invite; sends nothing. */
export function proposeMember(
	inviteBlob: string,
	name: string,
	surname: string,
	preferredHandles: string[]
): Promise<ProposedPersonaDto> {
	return invoke<ProposedPersonaDto>('propose_member', { inviteBlob, name, surname, preferredHandles });
}

/** LLR-q2k9z8: the provisional Member's admission request, only with `confirmed` true. */
export function requestAdmission(personaId: string, confirmed: boolean): Promise<AdmissionDto> {
	return invoke<AdmissionDto>('request_admission', { personaId, confirmed });
}

/** LLR-7n4hgr: record a revocation as a provisional change; no peer address. */
export function proposeRevocation(orgId: string, memberId: string): Promise<void> {
	return invoke<void>('propose_revocation', { orgId, memberId });
}

/** LLR-j2x9jb: verify every Organisation and provisional founding held; one entry per result. */
export function verifyOrganisations(): Promise<VerificationEntryDto[]> {
	return invoke<VerificationEntryDto[]>('verify_organisation');
}

/** verification-result: LLR-8grnbb, LLR-84cxpt. One per result of a verification the app ran itself. */
export function onVerificationResult(handler: (payload: VerificationEntryDto) => void): Promise<UnlistenFn> {
	return listen<VerificationEntryDto>('verification-result', (event) => handler(event.payload));
}

/** admission-requested: LLR-dp45mm. One per admission request kept on arrival. */
export function onAdmissionRequested(handler: (payload: AdmissionRequestDto) => void): Promise<UnlistenFn> {
	return listen<AdmissionRequestDto>('admission-requested', (event) => handler(event.payload));
}

/** LLR-k57uqv: the Person as `propose_person` returns it: its Device key (64 hex), the names stored, its status. */
export interface PersonDto {
	device_public_key: string;
	name: string;
	surname: string;
	status: 'provisional';
}

/** LLR-k57uqv: why `propose_person` refused, and org-io's text (a refused field named first). */
export interface PersonRefusalDto {
	kind: 'person_already_held' | 'refused';
	message: string;
}

/** LLR-k57uqv: propose the user's Person from a name and a surname (no Handle); it rejects with a `PersonRefusalDto`. */
export function proposePerson(name: string, surname: string): Promise<PersonDto> {
	return invoke<PersonDto>('propose_person', { name, surname });
}

/** LLR-6ff735: a proposed founding, with its founder's Device key (64 hex). */
export interface ProposedFoundingDto {
	persona_id: string;
	status: 'provisional_founding';
	founder: string;
}

/** LLR-6ff735: propose a founding; its result carries the founder's Device key the founding's write takes. */
export function proposeFoundingOrganisation(
	name: string,
	surname: string,
	handle: string,
	orgName: string
): Promise<ProposedFoundingDto> {
	return invoke<ProposedFoundingDto>('propose_founding_organisation', { name, surname, handle, orgName });
}

/** LLR-dt8ak9 (LLR-2u3cga): the updated Organisation org-io proposed; nothing is written. */
export interface ProposedOrganisationDto {
	epoch: number;
	root: string; // 64 hex chars
	org_public_key: string; // 64 hex chars
}

/** LLR-dt8ak9: propose the next epoch of `orgId` from the provisional changes org-io holds. */
export function proposeUpdatedOrganisation(orgId: string): Promise<ProposedOrganisationDto> {
	return invoke<ProposedOrganisationDto>('propose_updated_organisation', { orgId });
}

/** LLR-dt8ak9 (LLR-vqz4s6): cancel the outstanding Invite `inviteId` of `orgId`, removing its kept request. */
export function cancelInvite(orgId: string, inviteId: string): Promise<void> {
	return invoke<void>('cancel_invite', { orgId, inviteId });
}

/** LLR-rhc555, LLR-k4rhbs: a write's outcome, tagged by `result`; an included write carries its verification. */
export type WriteDto =
	| { result: 'included'; epoch: number; verification: VerificationEntryDto }
	| { result: 'refused'; reason: string }
	| { result: 'unknown'; reason: string };

/** LLR-rhc555: write a held Organisation by `orgId`, or a founding by its `founder`; exactly one. */
export function writeOrganisationToChain(target: { orgId: string } | { founder: string }): Promise<WriteDto> {
	return invoke<WriteDto>('write_organisation_to_chain', target);
}

/** Return the current connection status (chain env vars + data dir). */
export function connectionStatus(): Promise<ConnectionStatus> {
	return invoke<ConnectionStatus>('connection_status');
}

/**
 * Spawn a background tokio task that loops receive_and_self_delete_if_revoked
 * and emits Tauri events. Returns immediately.
 */
export function startReceiver(): Promise<void> {
	return invoke<void>('start_receiver');
}

// ---------------------------------------------------------------------------
// Events — 8 event types
// ---------------------------------------------------------------------------

/** membership-updated: emitted after a successful receive_and_verify (not revoked). */
export function onMembershipUpdated(
	cb: (payload: MembershipUpdatedPayload) => void
): Promise<UnlistenFn> {
	return listen<MembershipUpdatedPayload>('membership-updated', (e) => cb(e.payload));
}

/**
 * incoming-verified: same payload as membership-updated; alias for first-admission case.
 * UI can treat identically to membership-updated.
 */
export function onIncomingVerified(
	cb: (payload: MembershipUpdatedPayload) => void
): Promise<UnlistenFn> {
	return listen<MembershipUpdatedPayload>('incoming-verified', (e) => cb(e.payload));
}

/** revoked: emitted when receive_and_self_delete_if_revoked returns SelfDeleted. */
export function onRevoked(cb: (payload: RevokedPayload) => void): Promise<UnlistenFn> {
	return listen<RevokedPayload>('revoked', (e) => cb(e.payload));
}

/** epoch-changed: emitted on every successful verify (superset of the others). */
export function onEpochChanged(cb: (payload: EpochChangedPayload) => void): Promise<UnlistenFn> {
	return listen<EpochChangedPayload>('epoch-changed', (e) => cb(e.payload));
}

/**
 * receiver-error: REQ-kn5rtx / REQ-wu6z9p. Emitted when the receive path fails
 * for a reason that is NOT a verdict on any update — a chain read or transport
 * failure (`OrgNodeError::Chain`). Nothing verified and nothing failed to
 * verify, so this carries a message and no organisation, epoch, root or
 * verdict, and it must be rendered outside the verification log (REQ-wu6z9p).
 *
 * The loop keeps running unless the same failure is also terminal, in which
 * case `receiver-stopped` follows it (REQ-jfxah3).
 */
export function onReceiverError(cb: (payload: ReceiverErrorPayload) => void): Promise<UnlistenFn> {
	return listen<ReceiverErrorPayload>('receiver-error', (e) => cb(e.payload));
}

/** verification-failed: REQ-affyf5. An incoming update did not verify. */
export function onVerificationFailed(
	cb: (payload: VerificationFailedPayload) => void
): Promise<UnlistenFn> {
	return listen<VerificationFailedPayload>('verification-failed', (e) => cb(e.payload));
}

/**
 * record-unreadable: REQ-2k7ys4 / REQ-dp95pv. An update verified but the
 * organisation record could not be read back, so no epoch or root is known.
 */
export function onRecordUnreadable(
	cb: (payload: RecordUnreadablePayload) => void
): Promise<UnlistenFn> {
	return listen<RecordUnreadablePayload>('record-unreadable', (e) => cb(e.payload));
}

/** receiver-stopped: REQ-jfxah3. The receiver loop has exited, with a reason. */
export function onReceiverStopped(
	cb: (payload: ReceiverStoppedPayload) => void
): Promise<UnlistenFn> {
	return listen<ReceiverStoppedPayload>('receiver-stopped', (e) => cb(e.payload));
}
