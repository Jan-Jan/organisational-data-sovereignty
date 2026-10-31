// @vitest-environment jsdom
/**
 * SDD-xg4nj7: what the Admin is shown of admission requests and when the
 * admission may be proposed (LLR-47zvjn, LLR-e63yrs, LLR-5mw2dj,
 * LLR-dhtec7), and the invitee's error for an invite another Device used
 * (LLR-d8998k). Tested on the extracted decisions, and on the rendered page
 * over the faked Tauri boundary (./fake-tauri.ts).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent, within } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import { type AdmissionRequestDto, type AdmissionDto } from '../src/lib/api';
import {
	admissionConfirmation,
	admissionGate,
	fileAdmissionRequests,
	admissionRequestMessage
} from '../src/lib/notices';

const ORG = 'ab'.repeat(20);
const INVITE = '1c'.repeat(32);
const DEVICE = 'cd'.repeat(32);
const ISSUER = 'ee'.repeat(32);
const PERSONA = '77'.repeat(32);

function request(overrides: Partial<AdmissionRequestDto> = {}): AdmissionRequestDto {
	return {
		org_id: ORG,
		invite_id: INVITE,
		invitee_name: 'Bob Builder',
		name: 'Bob',
		surname: 'Builder',
		preferred_handles: ['bob', 'builder'],
		device_public_key: DEVICE,
		...overrides
	};
}

beforeEach(() => tauri.reset());

describe('admissionConfirmation', () => {
	// verifies: LLR-47zvjn
	it('admissionConfirmation shows the invitee name beside the request and marks a mismatch', () => {
		const shown = admissionConfirmation(request());
		expect(shown.invitee_name).toBe('Bob Builder');
		expect(shown.name).toBe('Bob');
		expect(shown.surname).toBe('Builder');
		expect(shown.device_public_key).toBe(DEVICE);
		expect(shown.device_public_key.length).toBe(64);
		expect(shown.mismatch).toBe(false);
		// Trimmed and case-folded: not a mismatch.
		expect(admissionConfirmation(request({ invitee_name: '  bob BUILDER ' })).mismatch).toBe(false);
		// Another name, or the parts joined otherwise: a mismatch.
		expect(admissionConfirmation(request({ invitee_name: 'Robert Builder' })).mismatch).toBe(true);
		expect(admissionConfirmation(request({ invitee_name: 'Bob  Builder' })).mismatch).toBe(true);
		expect(admissionConfirmation(request({ invitee_name: 'BobBuilder' })).mismatch).toBe(true);
	});
});

describe('admissionGate', () => {
	// verifies: LLR-e63yrs
	it('admissionGate refuses until confirmed and proposeAdmission is invoked with confirmed true', () => {
		const shown = admissionConfirmation(request());
		expect(admissionGate({ request: null, confirmed: true })).toEqual({
			ok: false,
			message: 'Choose the admission request to admit.'
		});
		expect(admissionGate({ request: shown, confirmed: false })).toEqual({
			ok: false,
			message: 'Confirm that this request is from the person you invited before admitting.'
		});
		expect(admissionGate({ request: shown, confirmed: true })).toEqual({ ok: true });
	});

	// verifies: LLR-e63yrs
	it('the Admit panel proposes nothing until the Admin confirms, then proposes once with confirmed true', async () => {
		tauri.answer('list_admission_requests', [request()]);
		tauri.answer('propose_admission', null);
		await openApp();
		await openTab('S3: Admit');

		// Nothing chosen yet: nothing to propose.
		expect(screen.queryByRole('button', { name: 'Propose admission' })).toBeNull();
		await fireEvent.click(screen.getByRole('button', { name: /Bob Builder — invite …1c1c1c1c/ }));

		// The confirmation: the typed invitee name beside the request's name, surname and DevicePublicKey.
		expect(screen.getByText('Invited as').nextElementSibling?.textContent).toBe('Bob Builder');
		expect(screen.getByText('Request').nextElementSibling?.textContent).toBe('Bob Builder');
		expect(screen.getByText('DevicePublicKey').nextElementSibling?.textContent).toBe(DEVICE);
		expect(screen.getByText('Confirm that this request is from the person you invited before admitting.')).toBeTruthy();

		// Unconfirmed: pressing Propose admission proposes nothing.
		const propose = screen.getByRole('button', { name: 'Propose admission' });
		await fireEvent.click(propose);
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([]);

		// Confirmed: proposed once, with the confirmation.
		await fireEvent.click(screen.getByRole('checkbox', { name: 'This request is from the person I invited' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Propose admission' }));
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([{ orgId: ORG, inviteId: INVITE, handle: '', confirmed: true }]);
		// The wording is LLR-b59vcx's (tests/epoch.commands.test.ts).
		expect(screen.getByText(/^Admission proposed\. /)).toBeTruthy();
	});
});

describe('the main page', () => {
	// verifies: LLR-5mw2dj
	it('the page lists kept requests once after it listens', async () => {
		const kept = request();
		const arriving = request({ invite_id: '2d'.repeat(32), invitee_name: 'Carol Jones', name: 'Carol', surname: 'Jones' });
		// While the page lists, one kept request is announced again and another arrives:
		// a page that listens first misses neither and shows each once.
		tauri.answer('list_admission_requests', () => {
			tauri.emit('admission-requested', kept);
			tauri.emit('admission-requested', arriving);
			return [kept];
		});
		await openApp();
		await openTab('S3: Admit');

		// Listed once, with no argument.
		expect(tauri.calls('list_admission_requests')).toEqual([undefined]);
		expect(screen.getAllByRole('button', { name: /Bob Builder — invite/ }).length).toBe(1);
		expect(screen.getAllByRole('button', { name: /Carol Jones — invite/ }).length).toBe(1);
	});

	// verifies: LLR-5mw2dj
	it('a refused subscription at start is shown on the page, not left unhandled', async () => {
		tauri.refuseListening('the event bus is closed');
		await openApp();
		expect(
			screen.getByText('listening for verification results and admission requests at start failed: the event bus is closed')
		).toBeTruthy();
		expect(screen.queryByText(/verification at start failed/)).toBeNull();
		// No listener started, so nothing was listed either.
		expect(tauri.calls('list_admission_requests')).toEqual([]);
	});
});

describe('fileAdmissionRequests', () => {
	// verifies: LLR-dhtec7
	it('a request listed and announced is shown once per org id and invite id', () => {
		const listed = request();
		const announced = request({ preferred_handles: ['bobby'] });
		const other = request({ invite_id: '2d'.repeat(32), invitee_name: 'Carol', name: 'Carol', surname: 'Jones' });
		// Listed, then announced: shown once, the later one replacing the earlier.
		const listedFirst = fileAdmissionRequests(fileAdmissionRequests([], [listed, other]), [announced]);
		expect(listedFirst).toEqual([admissionConfirmation(announced), admissionConfirmation(other)]);
		// Announced, then listed: shown once.
		const announcedFirst = fileAdmissionRequests(fileAdmissionRequests([], [announced]), [listed, other]);
		expect(announcedFirst.map((confirmation) => `${confirmation.org_id}/${confirmation.invite_id}`)).toEqual([`${ORG}/${INVITE}`, `${ORG}/${'2d'.repeat(32)}`]);
		expect(announcedFirst[0]).toEqual(admissionConfirmation(listed));
		// Each entry is built exactly as for an event with the same fields.
		expect(fileAdmissionRequests([], [listed])).toEqual([admissionConfirmation(listed)]);
		// Another Organisation with the same invite id is another request.
		expect(fileAdmissionRequests([], [listed, request({ org_id: 'cc'.repeat(20) })]).length).toBe(2);
	});
});

describe('admissionRequestMessage', () => {
	// verifies: LLR-d8998k
	it('an invite used by another device is an error message above the form', async () => {
		const message = admissionRequestMessage({ result: 'invite_used_by_another_device' });
		expect(message?.severity).toBe('error');
		expect(message?.text).toMatch(/another device already used this invite/i);
		expect(message?.text).toMatch(/contact the person who invited you/i);
		const others: AdmissionDto[] = [
			{ result: 'delivered' },
			{ result: 'already_received' },
			{ result: 'refused' },
			{ result: 'not_delivered', reason: 'timed out' }
		];
		for (const outcome of others) {
			expect(admissionRequestMessage(outcome), outcome.result).toBeNull();
		}

		// The invitee's panel shows that error as an alert, above the form.
		tauri.answer('read_invite', { org_name: 'Acme', org_id: ORG, invitee_name: 'Bob Builder', issuer_device_key: ISSUER });
		tauri.answer('propose_member', { persona_id: PERSONA, status: 'provisional_member' });
		tauri.answer('request_admission', { result: 'invite_used_by_another_device' });
		await openApp();
		await openTab('S2: Invite');
		await fireEvent.input(screen.getByLabelText('Paste Invite'), { target: { value: 'INVITE-BLOB' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Read Invite' }));
		await settle();
		await fireEvent.click(screen.getByRole('button', { name: 'Propose Member' }));
		await settle();
		await fireEvent.click(screen.getByRole('checkbox', { name: 'I understand' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission' }));
		await settle();

		const shown = screen.getByText('Another Device already used this invite. Contact the person who invited you for a new one.');
		expect(screen.getAllByRole('alert')).toContain(shown);
		const form = screen.getByLabelText('Paste Invite');
		expect(shown.compareDocumentPosition(form) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
	});

	// verifies: LLR-d8998k
	it('no other outcome of the request shows that error', async () => {
		tauri.answer('read_invite', { org_name: 'Acme', org_id: ORG, invitee_name: 'Bob Builder', issuer_device_key: ISSUER });
		tauri.answer('propose_member', { persona_id: PERSONA, status: 'provisional_member' });
		tauri.answer('request_admission', { result: 'refused' });
		await openApp();
		await openTab('S2: Invite');
		await fireEvent.input(screen.getByLabelText('Paste Invite'), { target: { value: 'INVITE-BLOB' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Read Invite' }));
		await settle();
		await fireEvent.click(screen.getByRole('button', { name: 'Propose Member' }));
		await settle();
		await fireEvent.click(screen.getByRole('checkbox', { name: 'I understand' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Request admission' }));
		await settle();
		expect(screen.getByText('The receiving Device refused the request.')).toBeTruthy();
		expect(screen.queryByText(/Another Device already used this invite/)).toBeNull();
		expect(within(document.body).queryAllByRole('alert').map((alert) => alert.textContent)).not.toContainEqual(
			expect.stringMatching(/contact the person who invited you/i)
		);
	});
});
