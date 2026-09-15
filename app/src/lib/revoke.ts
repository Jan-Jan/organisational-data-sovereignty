import type { TransportMode } from './api';

export interface RevokeInput {
	transportMode: TransportMode;
	memberIdHex: string;
	peerAddrBlob: string;
}

export type RevokeCheck = { ok: true } | { ok: false; message: string };

/**
 * REQ-vgr7s2, REQ-he8ejb. Whether a revocation may be submitted.
 *
 * The peer address is required ONLY in Loopback. In Networked the joiner is
 * reached by EndpointId via discovery and the join request carries no address
 * at all — so demanding one made revocation unreachable in the default
 * transport (HAZ-n97v5g).
 */
export function validateRevokeInput(input: RevokeInput): RevokeCheck {
	const memberId = input.memberIdHex.trim().replace(/^0x/, '');
	if (memberId.length !== 64) {
		return { ok: false, message: `Member ID must be 64 hex characters, got ${memberId.length}.` };
	}
	if (!/^[0-9a-fA-F]+$/.test(memberId)) {
		return { ok: false, message: 'Member ID must be hexadecimal.' };
	}
	if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {
		return {
			ok: false,
			message: 'Peer address is required in Loopback transport (same-machine dialling).'
		};
	}
	return { ok: true };
}
