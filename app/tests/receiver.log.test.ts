/**
 * Where one receiver announcement lands in the verification view.
 *
 * REQ-wu6z9p — a receiver error is rendered OUTSIDE the verification log and is
 *              assigned no verification outcome.
 * RC-3rddh7  — the control both this and REQ-kn5rtx (the Rust half) implement.
 *
 * The defect this guards: the receiver loop used to route every receive-path
 * error into `verification-failed`, and the component rendered that as a
 * "✗ MISMATCH" row under "Verified Updates (chain root match)". A transport
 * hiccup was shown to the operator as a root mismatch — a fresh instance of
 * HAZ-9fmhm4. The Rust half now classifies; this half must not undo that by
 * putting a receiver error back into the log.
 *
 * Every test here exercises the extracted decision, never a component: nothing
 * in this change renders `Membership.svelte` (not-minted control 1).
 */

import { describe, it, expect } from 'vitest';
import { applyReceiverEvent, type ReceiverView } from '../src/lib/verify';

const TS = '2026-09-14T11:00:00.000Z';
const ORG = 'aa'.repeat(20);
const ROOT = 'cc'.repeat(32);

const empty: ReceiverView = { verifyLog: [], receiverErrors: [] };

describe('applyReceiverEvent', () => {
	// verifies: LLR-fb7jp5
	it('produces no verification row for a receiver error', () => {
		const next = applyReceiverEvent(
			empty,
			{ kind: 'receiver-error', message: 'chain read failed: rpc timeout' },
			TS
		);
		expect(next.verifyLog).toEqual([]);
	});

	// verifies: LLR-fb7jp5
	it('renders a receiver error outside the verification log', () => {
		const next = applyReceiverEvent(
			empty,
			{ kind: 'receiver-error', message: 'chain read failed: rpc timeout' },
			TS
		);
		// The operator still sees it — the requirement is about WHERE, not about
		// swallowing the failure. The dead "Receiver Errors (non-fatal)" block
		// exists precisely for this.
		expect(next.receiverErrors).toHaveLength(1);
		expect(next.receiverErrors[0]).toContain('chain read failed: rpc timeout');
	});

	// verifies: LLR-fb7jp5
	it('leaves an existing verification log untouched', () => {
		const seeded = applyReceiverEvent(
			empty,
			{ kind: 'verified', org_id: ORG, epoch: 4, root: ROOT },
			TS
		);
		const after = applyReceiverEvent(
			seeded,
			{ kind: 'receiver-error', message: 'connection reset by peer' },
			TS
		);
		// Not merely "the same length": the same rows, unmodified. A receiver
		// error must not flip the verdict of a row already in the log either.
		expect(after.verifyLog).toEqual(seeded.verifyLog);
		expect(after.verifyLog.every((r) => r.verified)).toBe(true);
	});

	// verifies: LLR-fb7jp5
	it('assigns no verification outcome however many receiver errors arrive', () => {
		// Abnormal inputs: an empty message, a message that reads like a verdict,
		// and a repeat. None of them may create a row — in particular the second,
		// which is the shape a message-sniffing implementation would misfile.
		let view = empty;
		for (const message of ['', 'recomputed root does not match the on-chain root', '', 'io']) {
			view = applyReceiverEvent(view, { kind: 'receiver-error', message }, TS);
		}
		expect(view.verifyLog).toEqual([]);
		expect(view.receiverErrors).toHaveLength(4);
	});

	// verifies: LLR-fb7jp5
	it('does put a verification failure in the log and not in the receiver errors', () => {
		// Contrast. If every announcement were dropped from the log, the four
		// assertions above would hold and prove nothing about the separation.
		// Same function, a genuine verdict, the other destination.
		const next = applyReceiverEvent(
			empty,
			{ kind: 'failed', org_id: ORG, message: 'recomputed root does not match the on-chain root' },
			TS
		);
		expect(next.verifyLog).toHaveLength(1);
		expect(next.verifyLog[0].verified).toBe(false);
		expect(next.receiverErrors).toEqual([]);
	});

	// verifies: LLR-fb7jp5
	it('does not mutate the view it was given', () => {
		// The component assigns the result back into its `$state`; a function that
		// mutated in place would appear to work and would break the separation
		// under any future caller that kept the previous view.
		const before: ReceiverView = { verifyLog: [], receiverErrors: [] };
		applyReceiverEvent(before, { kind: 'receiver-error', message: 'io' }, TS);
		expect(before).toEqual({ verifyLog: [], receiverErrors: [] });
	});

	// verifies: LLR-rzx6ks, LLR-fb7jp5 (the `[<timestamp>] <message>` format)
	it('files the newest announcement first in each list', () => {
		let view = empty;
		view = applyReceiverEvent(view, { kind: 'verified', org_id: ORG, epoch: 1, root: ROOT }, TS);
		view = applyReceiverEvent(view, { kind: 'verified', org_id: ORG, epoch: 2, root: ROOT }, TS);
		view = applyReceiverEvent(view, { kind: 'receiver-error', message: 'first' }, TS);
		view = applyReceiverEvent(view, { kind: 'receiver-error', message: 'second' }, TS);
		expect(view.verifyLog.map((row) => row.epoch)).toEqual([2, 1]);
		expect(view.receiverErrors).toEqual([`[${TS}] second`, `[${TS}] first`]);
	});

	// verifies: LLR-rzx6ks
	it('keeps at most 50 verification rows, dropping the oldest', () => {
		// Boundary: the 51st row pushes out the first.
		let view = empty;
		for (let epoch = 0; epoch <= 50; epoch++) {
			view = applyReceiverEvent(view, { kind: 'verified', org_id: ORG, epoch, root: ROOT }, TS);
		}
		expect(view.verifyLog).toHaveLength(50);
		expect(view.verifyLog[0].epoch).toBe(50);
		expect(view.verifyLog[49].epoch).toBe(1);
	});

	// verifies: LLR-rzx6ks, LLR-fb7jp5 (the `[<timestamp>] <message>` format)
	it('keeps at most 20 receiver errors, dropping the oldest', () => {
		// Boundary: the 21st error pushes out the first.
		let view = empty;
		for (let i = 0; i <= 20; i++) {
			view = applyReceiverEvent(view, { kind: 'receiver-error', message: `e${i}` }, TS);
		}
		expect(view.receiverErrors).toHaveLength(20);
		expect(view.receiverErrors[0]).toBe(`[${TS}] e20`);
		expect(view.receiverErrors[19]).toBe(`[${TS}] e1`);
	});
});
