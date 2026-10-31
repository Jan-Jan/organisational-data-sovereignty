// @vitest-environment jsdom
/**
 * REQ-2yh3c8 (LLR-qcg6bz, LLR-prh433): before a provisional Member's first
 * admission request the user is told that nothing verified the Invite's
 * sender or the Organisation name it states, and that the request reveals
 * the name, surname and preferred Handles entered to the Device the Invite
 * names; the request is sent only once the user confirms, and a retry of a
 * confirmed request asks nothing again. Tested on the extracted decision, and
 * on the rendered invitee's panel over the faked Tauri boundary
 * (./fake-tauri.ts).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import { ADMISSION_REQUEST_WARNING, admissionRequestGate } from '../src/lib/invite';

const ORG = 'ab'.repeat(20);
const ISSUER = 'ee'.repeat(32);
const PERSONA = '77'.repeat(32);

beforeEach(() => tauri.reset());

/** The invitee reads an Invite and proposes a Member from it, up to the admission request. */
async function proposeMemberFromInvite() {
	tauri.answer('read_invite', { org_name: 'Acme', org_id: ORG, invitee_name: 'Bob Builder', issuer_device_key: ISSUER });
	tauri.answer('propose_member', { persona_id: PERSONA, status: 'provisional_member' });
	tauri.answer('request_admission', { result: 'not_delivered', reason: 'the Admin Device is offline' });
	await openApp();
	await openTab('S2: Invite');
	await fireEvent.input(screen.getByLabelText('Paste Invite'), { target: { value: 'INVITE-BLOB' } });
	await fireEvent.click(screen.getByRole('button', { name: 'Read Invite' }));
	await settle();
	await fireEvent.input(screen.getByLabelText('Name'), { target: { value: 'Bob' } });
	await fireEvent.input(screen.getByLabelText('Surname'), { target: { value: 'Builder' } });
	await fireEvent.input(screen.getByLabelText('Preferred Handles (1 to 3, comma-separated)'), {
		target: { value: 'bob, builder' }
	});
	await fireEvent.click(screen.getByRole('button', { name: 'Propose Member' }));
	await settle();
}

describe('the admission request warning and gate', () => {
	// verifies: LLR-qcg6bz
	it('states each thing the user must be told before the first admission request', () => {
		expect(ADMISSION_REQUEST_WARNING).toMatch(/nothing has verified who sent this invite/i);
		expect(ADMISSION_REQUEST_WARNING).toMatch(/organisation name/i);
		expect(ADMISSION_REQUEST_WARNING).toMatch(/name, surname and preferred handles/i);
		expect(ADMISSION_REQUEST_WARNING).toMatch(/device the invite names/i);
		expect(ADMISSION_REQUEST_WARNING).not.toMatch(/persona you choose/i);
	});

	// verifies: LLR-qcg6bz
	it('refuses until the user has confirmed', () => {
		expect(admissionRequestGate({ confirmedBefore: false, confirmed: false })).toEqual({
			ok: false,
			message: 'Read the warning and confirm before requesting admission.'
		});
	});

	// verifies: LLR-qcg6bz
	it('allows the request once the user confirmed', () => {
		expect(admissionRequestGate({ confirmedBefore: false, confirmed: true })).toEqual({ ok: true });
	});

	// verifies: LLR-qcg6bz
	it('the invitee is shown the warning and nothing is requested until the user confirms', async () => {
		await proposeMemberFromInvite();
		expect(tauri.calls('propose_member')).toEqual([
			{ inviteBlob: 'INVITE-BLOB', name: 'Bob', surname: 'Builder', preferredHandles: ['bob', 'builder'] }
		]);

		// The warning, before the first request, as an alert.
		expect(screen.getAllByRole('alert').map((alert) => alert.textContent)).toContainEqual(
			expect.stringContaining(ADMISSION_REQUEST_WARNING)
		);
		expect(screen.getByText('Read the warning and confirm before requesting admission.')).toBeTruthy();

		// Unconfirmed: pressing Request admission sends nothing.
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission' }));
		await settle();
		expect(tauri.calls('request_admission')).toEqual([]);

		// Confirmed: sent, with confirmed true.
		await fireEvent.click(screen.getByRole('checkbox', { name: 'I understand' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission' }));
		await settle();
		expect(tauri.calls('request_admission')).toEqual([{ personaId: PERSONA, confirmed: true }]);
	});

	// verifies: LLR-prh433
	it('a retry of a confirmed request asks nothing again', async () => {
		expect(admissionRequestGate({ confirmedBefore: true, confirmed: false })).toEqual({ ok: true });

		await proposeMemberFromInvite();
		await fireEvent.click(screen.getByRole('checkbox', { name: 'I understand' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission' }));
		await settle();
		expect(screen.getByText('Not delivered: the Admin Device is offline')).toBeTruthy();

		// The retry: no warning and no confirmation asked again, and it is sent with confirmed true.
		expect(screen.queryByText(ADMISSION_REQUEST_WARNING)).toBeNull();
		expect(screen.queryByRole('checkbox', { name: 'I understand' })).toBeNull();
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission again' }));
		await settle();
		expect(tauri.calls('request_admission')).toEqual([
			{ personaId: PERSONA, confirmed: true },
			{ personaId: PERSONA, confirmed: true }
		]);
	});
});
