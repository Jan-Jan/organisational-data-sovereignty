/**
 * What a verification log row is allowed to claim.
 *
 * REQ-a83vqr — a verification-failure event renders as a row whose outcome is
 *              "not verified".
 * REQ-akt4p7 — a row's outcome is derived from the event that produced it, and
 *              from no other source.
 *
 * Before this change `Membership.svelte` assigned the literal `verified: true`
 * at both of its only two assignment sites, so the ✗ MISMATCH branch was
 * unreachable by any input (HAZ-9fmhm4).
 */

import { describe, it, expect } from 'vitest';
import { verifyResultFrom } from '../src/lib/verify';

const TS = '2026-09-14T11:00:00.000Z';
const ORG = 'aa'.repeat(20);
const ROOT = 'cc'.repeat(32);

describe('verifyResultFrom', () => {
	// verifies: LLR-53hayh
	it('renders a verified event as verified', () => {
		expect(verifyResultFrom({ kind: 'verified', org_id: ORG, epoch: 7, root: ROOT }, TS)).toEqual({
			org_id: ORG,
			epoch: 7,
			root: ROOT,
			verified: true,
			detail: null,
			ts: TS
		});
	});

	// verifies: LLR-a4xwvj
	it('renders a failed event as not verified', () => {
		// The test that would have caught HAZ-9fmhm4: the ✗ state must be
		// reachable from an input, not merely present in the markup.
		const row = verifyResultFrom({ kind: 'failed', org_id: ORG, message: 'root mismatch' }, TS);
		expect(row.verified).toBe(false);
	});

	// verifies: LLR-a4xwvj
	it('gives a failed event no epoch and no root', () => {
		// null, not 0 and '': a failure knows neither, and 0 is genesis
		// (HAZ-5ha5vv). The same rule as REQ-tw4cb5 on the Rust side.
		const row = verifyResultFrom({ kind: 'failed', org_id: ORG, message: 'root mismatch' }, TS);
		expect(row.epoch).toBeNull();
		expect(row.root).toBeNull();
	});

	// verifies: LLR-mzae5q
	it('renders a placeholder rather than "null" when the failure names no organisation', () => {
		// REQ-affyf5's "where the failing update names one": the backend is
		// permitted to send null, so the row must read as prose, not as a literal.
		const row = verifyResultFrom({ kind: 'failed', org_id: null, message: 'decode failed' }, TS);
		expect(row.org_id).toBe('(unknown organisation)');
		expect(row.org_id).not.toBe('null');
	});

	// verifies: LLR-mzae5q
	it('keeps the organisation a failed event names', () => {
		// The placeholder stands in for an absent organisation only.
		const row = verifyResultFrom({ kind: 'failed', org_id: ORG, message: 'root mismatch' }, TS);
		expect(row.org_id).toBe(ORG);
	});

	// verifies: LLR-a4xwvj
	it('carries the failure message as the row detail', () => {
		const row = verifyResultFrom({ kind: 'failed', org_id: ORG, message: 'root mismatch' }, TS);
		expect(row.detail).toBe('root mismatch');
	});

	// verifies: LLR-53hayh
	it('gives a verified event no detail', () => {
		const row = verifyResultFrom({ kind: 'verified', org_id: ORG, epoch: 7, root: ROOT }, TS);
		expect(row.detail).toBeNull();
	});

	// verifies: LLR-53hayh
	it('preserves epoch 0 on a verified event rather than coercing it to null', () => {
		// Genesis is a real, reachable epoch. A row must be able to show 0 while a
		// failure shows nothing at all — the two absences are different.
		const row = verifyResultFrom({ kind: 'verified', org_id: ORG, epoch: 0, root: ROOT }, TS);
		expect(row.epoch).toBe(0);
		expect(row.epoch).not.toBeNull();
	});

	// verifies: LLR-53hayh, LLR-a4xwvj
	it('passes the timestamp through unmodified', () => {
		const odd = '   not a timestamp   ';
		expect(verifyResultFrom({ kind: 'verified', org_id: ORG, epoch: 1, root: ROOT }, odd).ts).toBe(
			odd
		);
		expect(verifyResultFrom({ kind: 'failed', org_id: null, message: 'x' }, odd).ts).toBe(odd);
	});
});
