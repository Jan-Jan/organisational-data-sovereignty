/**
 * SDD-xg4nj7: the notices the webview shows of a membership change in
 * flight. Pure functions; the components render what they return.
 */
import type {
	AdmissionRequestDto,
	AdmissionDto,
	DeviceReportDto,
	DiscardedDto,
	VerificationEntryDto
} from './api';

/** LLR-47zvjn: one kept admission request as the Admin is shown it. */
export interface AdmissionConfirmation extends AdmissionRequestDto {
	/** The typed invitee name differs from the request's name and surname. */
	mismatch: boolean;
}

/**
 * LLR-47zvjn: the invitee name typed for the Invite beside the request's
 * name, surname and Device key in full; a mismatch when the trimmed,
 * case-folded invitee name differs from the name and surname joined by one
 * space (case-folded).
 */
export function admissionConfirmation(request: AdmissionRequestDto): AdmissionConfirmation {
	const typed = request.invitee_name.trim().toLowerCase();
	const stated = `${request.name} ${request.surname}`.toLowerCase();
	return { ...request, preferred_handles: [...request.preferred_handles], mismatch: typed !== stated };
}

export type AdmissionCheck = { ok: true } | { ok: false; message: string };

/** LLR-e63yrs: refuses, naming what is missing, until the Admin confirmed the request shown. */
export function admissionGate(input: { request: AdmissionConfirmation | null; confirmed: boolean }): AdmissionCheck {
	if (input.request === null) {
		return { ok: false, message: 'Choose the admission request to admit.' };
	}
	if (!input.confirmed) {
		return { ok: false, message: 'Confirm that this request is from the person you invited before admitting.' };
	}
	return { ok: true };
}

/**
 * LLR-dhtec7: the Admin's admission requests, one confirmation per
 * `org_id` and `invite_id`. A request both listed and announced, in either
 * order, is shown once; a later one with the same pair replaces the
 * earlier in its place; a new pair is added at the end.
 */
export function fileAdmissionRequests(
	shown: AdmissionConfirmation[],
	requests: AdmissionRequestDto[]
): AdmissionConfirmation[] {
	const next = [...shown];
	for (const request of requests) {
		const confirmation = admissionConfirmation(request);
		const at = next.findIndex((held) => held.org_id === request.org_id && held.invite_id === request.invite_id);
		if (at === -1) next.push(confirmation);
		else next[at] = confirmation;
	}
	return next;
}

export interface RequestMessage {
	severity: 'error';
	text: string;
}

/** LLR-d8998k: an error for an invite another Device used; nothing for any other outcome. */
export function admissionRequestMessage(outcome: AdmissionDto): RequestMessage | null {
	if (outcome.result === 'invite_used_by_another_device') {
		return {
			severity: 'error',
			text: 'Another Device already used this invite. Contact the person who invited you for a new one.'
		};
	}
	return null;
}

/** LLR-k5rf3r: one discarded change of an adopted epoch, kept until dismissed. */
export interface DiscardedNotice {
	id: string;
	org_id: string;
	epoch: number;
	change: DiscardedDto;
	text: string;
}

function discardedText(orgId: string, epoch: number, change: DiscardedDto): string {
	const where = `epoch ${epoch} of …${orgId.slice(-12)}`;
	if (change.kind === 'admission') {
		const who = [change.name, change.surname].filter((part) => part !== null).join(' ') || 'the invitee';
		return `The admission of ${who} (Handle ${change.handle}) was left out of ${where}.`;
	}
	const member = change.handle === null ? change.member_id : `${change.handle} (${change.member_id})`;
	return `The revocation of Member ${member} was left out of ${where}.`;
}

/**
 * LLR-k5rf3r: one notice per `discarded` change of an `adopted` result,
 * naming the Member and the kind of change, added after those kept. No
 * count limit drops one, and no later result removes one. An adopted entry
 * names its Organisation (only a founding with no proxy has none).
 */
export function applyDiscarded(notices: DiscardedNotice[], entry: VerificationEntryDto): DiscardedNotice[] {
	if (entry.result !== 'adopted' || entry.org_id === null) return notices;
	const orgId = entry.org_id;
	const added = entry.discarded.map((change, index) => ({
		id: `${orgId}/${entry.epoch}/${index}`,
		org_id: orgId,
		epoch: entry.epoch,
		change,
		text: discardedText(orgId, entry.epoch, change)
	}));
	return [...notices, ...added];
}

/** LLR-k5rf3r: removes that one notice, and only it. */
export function dismissNotice(notices: DiscardedNotice[], id: string): DiscardedNotice[] {
	return notices.filter((notice) => notice.id !== id);
}

export type AdmissionReproposal = { command: 'propose_admission'; org_id: string; invite_id: string; handle: string };
export type RevocationReproposal = { command: 'propose_revocation'; org_id: string; member_id: string };
export type Reproposal = AdmissionReproposal | RevocationReproposal;

/** LLR-md6hhz: the command that proposes the notice's change again; the notice is not touched. */
export function reproposal(notice: DiscardedNotice): Reproposal {
	return notice.change.kind === 'admission'
		? { command: 'propose_admission', org_id: notice.org_id, invite_id: notice.change.invite_id, handle: notice.change.handle }
		: { command: 'propose_revocation', org_id: notice.org_id, member_id: notice.change.member_id };
}

/** What "Propose again" does first (LLR-md6hhz, LLR-e63yrs). */
export type ReproposalStep =
	| { step: 'confirm'; confirmation: AdmissionConfirmation; reproposal: AdmissionReproposal }
	| { step: 'propose'; reproposal: RevocationReproposal }
	| { step: 'unavailable'; message: string };

/**
 * LLR-md6hhz, LLR-e63yrs: an admission is proposed again only through the
 * REQ-88npkt confirmation, beside the kept request for its Organisation
 * and invite (the confirmation the Admin was first shown); with no such
 * request kept nothing can be confirmed, so nothing is proposed. A
 * revocation has no confirmation to repeat. The notice is not touched.
 */
export function reproposalStep(notice: DiscardedNotice, requests: AdmissionConfirmation[]): ReproposalStep {
	const again = reproposal(notice);
	if (again.command === 'propose_revocation') return { step: 'propose', reproposal: again };
	const confirmation = requests.find((held) => held.org_id === again.org_id && held.invite_id === again.invite_id);
	if (confirmation === undefined) {
		return {
			step: 'unavailable',
			message: 'The admission request for this invite is no longer kept, so it cannot be confirmed again.'
		};
	}
	return { step: 'confirm', confirmation, reproposal: again };
}

/** LLR-fsk4g2: every Device's outcome of the single send, in the report's order; report only, no action. */
export function sendOutcomes(entry: VerificationEntryDto): DeviceReportDto[] {
	return entry.result === 'adopted' && entry.sends ? [...entry.sends] : [];
}

/**
 * LLR-9r6y7p: one send report: each Device's outcome of an own adoption's
 * send, or the refusal of a whole send or request with org-io's reason.
 */
export type SendReport = { id: string; org_id: string; title: string } & (
	| { kind: 'sends'; outcomes: DeviceReportDto[] }
	| { kind: 'refused'; reason: string }
);

/**
 * LLR-9r6y7p: adds one report for an entry that carries the outcome of a
 * send (`sends` or `sends_refused`) or the refusal of a request
 * (`requests_refused`), after those kept; any other entry adds none. No
 * later report replaces an earlier one.
 */
export function applySendReports(reports: SendReport[], entry: VerificationEntryDto): SendReport[] {
	if (entry.org_id === null) return reports;
	const org = `…${entry.org_id.slice(-12)}`;
	// Unique while the list lives: one past the highest id held, so a
	// dismissal never lets a later report take a kept one's id.
	const id = String(reports.reduce((highest, report) => Math.max(highest, Number(report.id)), -1) + 1);
	if (entry.result === 'adopted' && entry.sends_refused !== undefined) {
		const title = `The send of epoch ${entry.epoch} of ${org} was refused: ${entry.sends_refused}`;
		return [...reports, { id, org_id: entry.org_id, title, kind: 'refused', reason: entry.sends_refused }];
	}
	if (entry.result === 'adopted' && entry.sends !== undefined) {
		const title = `Sent epoch ${entry.epoch} of ${org} to each Device:`;
		return [...reports, { id, org_id: entry.org_id, title, kind: 'sends', outcomes: sendOutcomes(entry) }];
	}
	if (entry.result === 'behind' && entry.requests_refused !== undefined) {
		const title = `The request of ${org} from its Members (chain at epoch ${entry.chain_epoch}) was refused: ${entry.requests_refused}`;
		return [...reports, { id, org_id: entry.org_id, title, kind: 'refused', reason: entry.requests_refused }];
	}
	return reports;
}

/** LLR-9r6y7p: removes that one report, and only it. */
export function dismissSendReport(reports: SendReport[], id: string): SendReport[] {
	return reports.filter((report) => report.id !== id);
}

/** LLR-e7bbm5: the admission requests without the one whose invite was cancelled. */
export function withdrawAdmissionRequest(
	shown: AdmissionConfirmation[],
	orgId: string,
	inviteId: string
): AdmissionConfirmation[] {
	return shown.filter((held) => !(held.org_id === orgId && held.invite_id === inviteId));
}
