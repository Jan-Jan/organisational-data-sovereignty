/**
 * Preconditions for submitting a revocation.
 *
 * REQ-he8ejb — the member id is 64 hex characters, `0x` prefix optional.
 *
 * Every test here exercises the extracted decision, not the component that
 * calls it.
 */

import { describe, it, expect } from 'vitest';
import { validateRevokeInput } from '../src/lib/revoke';

const MEMBER_ID = 'bb'.repeat(32);

describe('validateRevokeInput', () => {
	// verifies: LLR-csbs5v
	it('rejects a 63-character member id', () => {
		const result = validateRevokeInput({
			memberIdHex: 'b'.repeat(63)
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('63');
	});

	// verifies: LLR-csbs5v
	it('rejects a 65-character member id', () => {
		const result = validateRevokeInput({
			memberIdHex: 'b'.repeat(65)
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('65');
	});

	// verifies: LLR-csbs5v
	it('rejects an empty member id', () => {
		const result = validateRevokeInput({
			memberIdHex: ''
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('0');
	});

	// verifies: LLR-csbs5v
	it('rejects a 64-character non-hexadecimal member id', () => {
		// Right width, wrong alphabet: the width check alone would admit it.
		const result = validateRevokeInput({
			memberIdHex: 'g'.repeat(64)
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toMatch(/hexadecimal/i);
	});

	// verifies: LLR-csbs5v
	it('accepts a 0x-prefixed member id', () => {
		// 66 characters on the wire, 64 after the prefix is stripped.
		expect(
			validateRevokeInput({
				memberIdHex: `0x${MEMBER_ID}`
			})
		).toEqual({ ok: true });
	});

	// verifies: LLR-csbs5v
	it('rejects a doubled 0x prefix', () => {
		// One optional prefix, not a run of them: the second `0x` stays and is
		// counted, so the id is 66 characters long and refused.
		const result = validateRevokeInput({
			memberIdHex: `0x0x${MEMBER_ID}`
		});
		expect(result.ok).toBe(false);
		expect(result.ok === false && result.message).toContain('66');
	});

	// verifies: LLR-csbs5v
	it('trims surrounding whitespace before measuring', () => {
		expect(validateRevokeInput({ memberIdHex: `  ${MEMBER_ID}\n` })).toEqual({ ok: true });
	});

	// verifies: LLR-csbs5v
	it('accepts a 64-character hexadecimal member id', () => {
		expect(validateRevokeInput({ memberIdHex: MEMBER_ID })).toEqual({ ok: true });
	});
});
