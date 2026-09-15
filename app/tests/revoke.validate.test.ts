/**
 * Preconditions for submitting a revocation.
 *
 * REQ-vgr7s2 — the peer address is required only in Loopback transport.
 * REQ-he8ejb — the member id is 64 hex characters, `0x` prefix optional.
 *
 * Every test here exercises the extracted decision, never a component: nothing
 * in this change renders `Revoke.svelte` (not-minted control 1).
 */

import { describe, it, expect } from 'vitest';
import { validateRevokeInput } from '../src/lib/revoke';

const MEMBER_ID = 'bb'.repeat(32);

describe('validateRevokeInput', () => {
	// verifies: REQ-vgr7s2
	it('accepts an empty peer address in networked transport', () => {
		// This is HAZ-n97v5g. Before this change the form refused unconditionally,
		// and a Networked join request never carries an address to supply — so
		// revocation could not be submitted at all in the shipped configuration.
		expect(
			validateRevokeInput({
				transportMode: 'networked',
				memberIdHex: MEMBER_ID,
				peerAddrBlob: ''
			})
		).toEqual({ ok: true });
	});

	// verifies: REQ-vgr7s2
	it('accepts a supplied peer address in networked transport', () => {
		expect(
			validateRevokeInput({
				transportMode: 'networked',
				memberIdHex: MEMBER_ID,
				peerAddrBlob: 'deadbeef'
			})
		).toEqual({ ok: true });
	});

	// verifies: REQ-vgr7s2
	it('rejects an empty peer address in loopback transport', () => {
		const result = validateRevokeInput({
			transportMode: 'loopback',
			memberIdHex: MEMBER_ID,
			peerAddrBlob: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toMatch(/loopback/i);
	});

	// verifies: REQ-vgr7s2
	it('accepts a supplied peer address in loopback transport', () => {
		expect(
			validateRevokeInput({
				transportMode: 'loopback',
				memberIdHex: MEMBER_ID,
				peerAddrBlob: 'deadbeef'
			})
		).toEqual({ ok: true });
	});

	// verifies: REQ-he8ejb
	it('rejects a 63-character member id', () => {
		const result = validateRevokeInput({
			transportMode: 'networked',
			memberIdHex: 'b'.repeat(63),
			peerAddrBlob: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('63');
	});

	// verifies: REQ-he8ejb
	it('rejects a 65-character member id', () => {
		const result = validateRevokeInput({
			transportMode: 'networked',
			memberIdHex: 'b'.repeat(65),
			peerAddrBlob: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('65');
	});

	// verifies: REQ-he8ejb
	it('rejects an empty member id', () => {
		const result = validateRevokeInput({
			transportMode: 'networked',
			memberIdHex: '',
			peerAddrBlob: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('0');
	});

	// verifies: REQ-he8ejb
	it('rejects a 64-character non-hexadecimal member id', () => {
		// Right width, wrong alphabet: the width check alone would admit it.
		const result = validateRevokeInput({
			transportMode: 'networked',
			memberIdHex: 'g'.repeat(64),
			peerAddrBlob: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toMatch(/hexadecimal/i);
	});

	// verifies: REQ-he8ejb
	it('accepts a 0x-prefixed member id', () => {
		// 66 characters on the wire, 64 after the prefix is stripped.
		expect(
			validateRevokeInput({
				transportMode: 'networked',
				memberIdHex: `0x${MEMBER_ID}`,
				peerAddrBlob: ''
			})
		).toEqual({ ok: true });
	});

	// verifies: REQ-he8ejb
	it('trims surrounding whitespace before measuring', () => {
		expect(
			validateRevokeInput({
				transportMode: 'networked',
				memberIdHex: `  ${MEMBER_ID}\n`,
				peerAddrBlob: ''
			})
		).toEqual({ ok: true });
		// And a whitespace-only address is an absent address, not a present one.
		expect(
			validateRevokeInput({
				transportMode: 'loopback',
				memberIdHex: MEMBER_ID,
				peerAddrBlob: '   '
			}).ok
		).toBe(false);
	});
});
