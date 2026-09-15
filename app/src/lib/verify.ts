export type VerifyEvent =
	| { kind: 'verified'; org_id: string; epoch: number; root: string }
	| { kind: 'failed'; org_id: string | null; message: string };

export interface VerifyRow {
	org_id: string;
	epoch: number | null;
	root: string | null;
	verified: boolean;
	detail: string | null;
	ts: string;
}

/**
 * REQ-a83vqr, REQ-akt4p7. The row's outcome comes from the event that produced
 * it and from nowhere else.
 *
 * Before this change the component assigned `verified: true` at both of its
 * assignment sites, so the ✗ state was unreachable by any input (HAZ-9fmhm4).
 *
 * `epoch` and `root` are `null` on a failure rather than `0` and `''` — the
 * same rule as REQ-tw4cb5 on the Rust side, applied on this one. A failure
 * knows neither, and 0 is genesis (HAZ-5ha5vv).
 */
export function verifyResultFrom(event: VerifyEvent, ts: string): VerifyRow {
	if (event.kind === 'verified') {
		return {
			org_id: event.org_id,
			epoch: event.epoch,
			root: event.root,
			verified: true,
			detail: null,
			ts
		};
	}
	return {
		org_id: event.org_id ?? '(unknown organisation)',
		epoch: null,
		root: null,
		verified: false,
		detail: event.message,
		ts
	};
}

/** The two lists the verification view keeps, and the only two destinations. */
export interface ReceiverView {
	verifyLog: VerifyRow[];
	receiverErrors: string[];
}

/**
 * Everything the receiver announces that this view files somewhere: the two
 * verification outcomes, and the receiver error that is not one.
 */
export type ReceiverAnnouncement = VerifyEvent | { kind: 'receiver-error'; message: string };

const MAX_VERIFY_ROWS = 50;
const MAX_RECEIVER_ERRORS = 20;

/**
 * REQ-wu6z9p / RC-3rddh7. Where one receiver announcement lands.
 *
 * A receiver error is a chain read or transport failure — no update verified
 * and none failed to verify — so it goes to `receiverErrors`, which the view
 * renders under "Receiver Errors (non-fatal)", and it produces NO row in the
 * verification log. The alternative, the one this replaces, showed a transport
 * hiccup to the operator as a "✗ MISMATCH" row under "Verified Updates (chain
 * root match)" — HAZ-9fmhm4, the verification display misrepresenting the
 * verification state.
 *
 * Returns a new view; it does not mutate the one it was given.
 */
export function applyReceiverEvent(
	view: ReceiverView,
	event: ReceiverAnnouncement,
	ts: string
): ReceiverView {
	if (event.kind === 'receiver-error') {
		return {
			verifyLog: view.verifyLog,
			receiverErrors: [
				`[${ts}] ${event.message}`,
				...view.receiverErrors.slice(0, MAX_RECEIVER_ERRORS - 1)
			]
		};
	}
	return {
		verifyLog: [
			verifyResultFrom(event, ts),
			...view.verifyLog.slice(0, MAX_VERIFY_ROWS - 1)
		],
		receiverErrors: view.receiverErrors
	};
}
