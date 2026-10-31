export interface RevokeInput {
	memberIdHex: string;
}

export type RevokeCheck = { ok: true } | { ok: false; message: string };

/**
 * SDD-czjf2c, REQ-he8ejb (LLR-csbs5v). Whether a revocation may be proposed, decided
 * from the member identifier alone: 64 hexadecimal characters after one
 * optional `0x`, surrounding whitespace trimmed. No peer address is asked
 * in any mode: org-io holds each Device's addresses.
 */
export function validateRevokeInput(input: RevokeInput): RevokeCheck {
	const memberId = input.memberIdHex.trim().replace(/^0x/, '');
	if (memberId.length !== 64) {
		return { ok: false, message: `Member ID must be 64 hex characters, got ${memberId.length}.` };
	}
	if (!/^[0-9a-fA-F]+$/.test(memberId)) {
		return { ok: false, message: 'Member ID must be hexadecimal.' };
	}
	return { ok: true };
}
