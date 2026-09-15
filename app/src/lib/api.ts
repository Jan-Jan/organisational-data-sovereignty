/**
 * Typed wrappers around Tauri invoke/listen for all 12 ODS commands + 8 events.
 *
 * Command signatures mirror commands.rs exactly (camelCase arg names per Tauri convention).
 * Return shapes mirror the Rust DTOs (PersonaDto, OrgDto, JoinRequestDto, ConnectionStatus).
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ---------------------------------------------------------------------------
// DTOs (mirror Rust structs)
// ---------------------------------------------------------------------------

export interface PersonaDto {
	persona_id: string;
	org_id: string | null;
	handle: string;
	name: string;
	surname: string;
	/** "Proposed" | "Active" | "Revoked" (Debug format of enum) */
	status: string;
}

export interface OrgDto {
	org_id: string;    // 40 hex chars
	epoch: number;
	root_hash: string; // 64 hex chars
	member_count: number;
}

export interface JoinRequestDto {
	handle: string;
	name: string;
	surname: string;
	member_key: string;     // hex
	device_key: string;     // hex
	has_node_addr: boolean;
	node_addr_blob: string; // hex — pass back to admit_member
}

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

export interface RevokedPayload {
	org_id: string;
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
	org_id: string;
}

/** REQ-jfxah3: the receiver loop has exited and is no longer running. */
export interface ReceiverStoppedPayload {
	reason: string;
}

// ---------------------------------------------------------------------------
// Commands — all 12
// ---------------------------------------------------------------------------

/** Create a new persona (no chain interaction). Returns persona_id. */
export function createPersona(handle: string, name: string, surname: string): Promise<string> {
	return invoke<string>('create_persona', { handle, name, surname });
}

/**
 * Create an organisation: build genesis trie + submit to chain.
 * Returns org_id (40 hex chars).
 */
export function createOrganisation(personaId: string): Promise<string> {
	return invoke<string>('create_organisation', { personaId });
}

/** Export an invite blob for the given org_id. */
export function exportInvite(orgId: string): Promise<string> {
	return invoke<string>('export_invite', { orgId });
}

/**
 * Import an invite blob; returns the org_id it's for (40 hex chars).
 * Also stores the invite locally.
 */
export function importInvite(blob: string): Promise<string> {
	return invoke<string>('import_invite', { blob });
}

/** Export a join-request blob for the given persona_id. */
export function exportJoinRequest(personaId: string): Promise<string> {
	return invoke<string>('export_join_request', { personaId });
}

/**
 * Decode and return the fields of a join-request blob (no persistence).
 * Returns JoinRequestDto.
 */
export function importJoinRequest(blob: string): Promise<JoinRequestDto> {
	return invoke<JoinRequestDto>('import_join_request', { blob });
}

/**
 * Admit a new member from a join-request blob.
 * org_secret_hex is optional (pass null if not needed).
 * Returns the new member_id as 64 hex chars.
 */
export function admitMember(
	orgId: string,
	joinRequestBlob: string,
	orgSecretHex: string | null = null
): Promise<string> {
	return invoke<string>('admit_member', { orgId, joinRequestBlob, orgSecretHex });
}

/**
 * Revoke a member by member_id_hex (64 hex chars) and peer_addr_blob (hex-encoded
 * postcard bytes of the iroh EndpointAddr from the original join request).
 *
 * REQ-vgr7s2: `peerAddrBlob` may be the empty string, which means "no address".
 * A Networked join request carries no address, so demanding one made revocation
 * unreachable in the default transport (HAZ-n97v5g). See `validateRevokeInput`.
 */
export function revokeMember(
	orgId: string,
	memberIdHex: string,
	peerAddrBlob: string
): Promise<void> {
	return invoke<void>('revoke_member', { orgId, memberIdHex, peerAddrBlob });
}

/** List all local personas (no key material returned). */
export function listPersonas(): Promise<PersonaDto[]> {
	return invoke<PersonaDto[]>('list_personas');
}

/** List all local org records. */
export function listOrgs(): Promise<OrgDto[]> {
	return invoke<OrgDto[]>('list_orgs');
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
