/**
 * LLR-8krgzj: the webview's admitMember passes admit_member exactly the org
 * id, the reply and the peer address — no Organisation secret.
 * (`Admit.svelte` passing no fourth argument is enforced by `npm run check`:
 * svelte-check refuses a call with more arguments than the function takes.)
 */

import { describe, it, expect, vi } from 'vitest';

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn(async () => '00'.repeat(32)) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import { admitMember } from '../src/lib/api';

describe('admitMember', () => {
	// verifies: LLR-8krgzj
	it('invokes admit_member with the org id, the reply and the peer address alone', async () => {
		await admitMember('01'.repeat(20), 'reply', 'addr');
		expect(invoke).toHaveBeenCalledWith('admit_member', {
			orgId: '01'.repeat(20),
			replyBlob: 'reply',
			peerAddrBlob: 'addr'
		});
		expect(admitMember.length).toBe(3);
	});
});
