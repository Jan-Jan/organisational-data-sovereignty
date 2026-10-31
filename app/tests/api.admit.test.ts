/**
 * LLR-gmb4p8: the webview's proposeAdmission passes propose_admission
 * exactly the org id, the invite id, the handle and the confirmation — no
 * Organisation secret, reply or peer address.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// The faked Tauri boundary every webview test uses, under backend.json's contract:
// its propose_admission answer is the fixture's `recorded`.
vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { tauri } from './fake-tauri';
import { proposeAdmission } from '../src/lib/api';

beforeEach(() => tauri.reset());

describe('proposeAdmission', () => {
	// verifies: LLR-gmb4p8
	it('invokes propose_admission with the org id, the invite id, the handle and the confirmation alone', async () => {
		tauri.answer('propose_admission', null);
		await proposeAdmission('01'.repeat(20), '02'.repeat(32), 'bob', true);
		expect(tauri.invoke).toHaveBeenCalledWith('propose_admission', {
			orgId: '01'.repeat(20),
			inviteId: '02'.repeat(32),
			handle: 'bob',
			confirmed: true
		});
		expect(proposeAdmission.length).toBe(4);
	});
});
