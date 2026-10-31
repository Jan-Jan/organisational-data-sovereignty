// @vitest-environment jsdom
/**
 * SDD-xg4nj7: the notices for discarded changes, their re-proposal (an
 * admission through the REQ-88npkt confirmation again) and the send
 * outcomes of an adopted epoch (LLR-k5rf3r, LLR-md6hhz, LLR-e63yrs,
 * LLR-fsk4g2). Tested on the extracted decisions, and on the rendered page
 * over the faked Tauri boundary (./fake-tauri.ts).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent, within } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import type { AdmissionRequestDto, VerificationEntryDto } from '../src/lib/api';
import {
	admissionConfirmation,
	applyDiscarded,
	dismissNotice,
	reproposal,
	sendOutcomes
} from '../src/lib/notices';

const sources = import.meta.glob('../src/**/*.{ts,svelte}', { query: '?raw', import: 'default', eager: true }) as Record<
	string,
	string
>;

const ORG = 'ab'.repeat(20);
const INVITE = '1c'.repeat(32);
const MEMBER = '2d'.repeat(32);
const DEVICE = 'cd'.repeat(32);

function adopted(epoch: number, discarded: Extract<VerificationEntryDto, { result: 'adopted' }>['discarded']): VerificationEntryDto {
	// Another Admin's epoch adopted here (`own` false): an own adoption also carries `sends` (backend.json).
	return { result: 'adopted', org_id: ORG, epoch, own: false, discarded };
}

const admission = { kind: 'admission' as const, invite_id: INVITE, name: 'Bob', surname: 'Builder', handle: 'bob' };
const revocation = { kind: 'revocation' as const, member_id: MEMBER, handle: 'carol' };

/** The kept admission request the discarded admission was for. */
function keptRequest(overrides: Partial<AdmissionRequestDto> = {}): AdmissionRequestDto {
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

/**
 * Opens the Membership panel with `kept` listed at start, then the backend
 * announces an adopted epoch that left out `discarded`.
 */
async function openNotices(kept: AdmissionRequestDto[], discarded: Parameters<typeof adopted>[1]) {
	tauri.answer('list_admission_requests', kept);
	tauri.answer('propose_admission', null);
	tauri.answer('propose_revocation', null);
	const page = await openApp();
	await openTab('S4: Membership');
	await settle();
	tauri.emit('verification-result', adopted(3, discarded));
	await settle();
	return page;
}

/** The notice whose text matches `text`, to act within it as the Admin does. */
function notice(text: RegExp) {
	return within(screen.getByText(text).parentElement!);
}

const ADMISSION_NOTICE = /The admission of Bob Builder \(Handle bob\) was left out of epoch 3/;
const REVOCATION_NOTICE = /The revocation of Member carol .* was left out of epoch 3/;

beforeEach(() => tauri.reset());

describe('applyDiscarded', () => {
	// verifies: LLR-k5rf3r
	it('applyDiscarded adds one notice per change and keeps it until dismissed', () => {
		const first = applyDiscarded([], adopted(3, [admission, revocation]));
		expect(first.length).toBe(2);
		expect(first[0].text).toMatch(/admission/i);
		expect(first[0].text).toContain('Bob Builder');
		expect(first[0].text).toContain('bob');
		expect(first[1].text).toMatch(/revocation/i);
		expect(first[1].text).toContain('carol');
		expect(new Set(first.map((notice) => notice.id)).size).toBe(2);
		// A later result, of any kind, removes none.
		let notices = first;
		for (const later of [
			adopted(4, []),
			{ result: 'in_step', org_id: ORG, epoch: 4 } as VerificationEntryDto,
			{ result: 'removed', org_id: ORG } as VerificationEntryDto
		]) {
			notices = applyDiscarded(notices, later);
		}
		expect(notices).toEqual(first);
		// No count limit drops a notice.
		let many = notices;
		for (let epoch = 5; epoch < 105; epoch++) many = applyDiscarded(many, adopted(epoch, [revocation]));
		expect(many.length).toBe(102);
		expect(many.slice(0, 2)).toEqual(first);
		// Only dismissNotice removes one, and only that one.
		const after = dismissNotice(many, first[0].id);
		expect(after.length).toBe(101);
		expect(after.some((notice) => notice.id === first[0].id)).toBe(false);
		expect(after[0]).toEqual(first[1]);
		// A change whose name or Handle org-io no longer holds still gets its notice.
		const unnamed = applyDiscarded([], adopted(6, [{ ...admission, name: null, surname: null }, { ...revocation, handle: null }]));
		expect(unnamed.length).toBe(2);
		expect(unnamed[0].text).toMatch(/admission/i);
		expect(unnamed[1].text).toContain(MEMBER);
	});
});

describe('reproposal', () => {
	// verifies: LLR-md6hhz
	it('reproposal names propose_admission or propose_revocation and the notice stays', async () => {
		const notices = applyDiscarded([], adopted(3, [admission, revocation]));
		expect(reproposal(notices[0])).toEqual({ command: 'propose_admission', org_id: ORG, invite_id: INVITE, handle: 'bob' });
		expect(reproposal(notices[1])).toEqual({ command: 'propose_revocation', org_id: ORG, member_id: MEMBER });
		// Proposing again leaves the notices as they were.
		const before = structuredClone(notices);
		reproposal(notices[0]);
		reproposal(notices[1]);
		expect(notices).toEqual(before);

		// On the panel: proposing either change again leaves both notices shown.
		await openNotices([keptRequest()], [admission, revocation]);
		await fireEvent.click(notice(REVOCATION_NOTICE).getByRole('button', { name: 'Propose again' }));
		await settle();
		expect(tauri.calls('propose_revocation')).toEqual([{ orgId: ORG, memberId: MEMBER }]);
		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		await fireEvent.click(screen.getByRole('checkbox', { name: 'This request is from the person I invited' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Propose admission again' }));
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([{ orgId: ORG, inviteId: INVITE, handle: 'bob', confirmed: true }]);
		expect(screen.getByText(ADMISSION_NOTICE)).toBeTruthy();
		expect(screen.getByText(REVOCATION_NOTICE)).toBeTruthy();
		// Only Dismiss removes a notice, and only that one.
		await fireEvent.click(notice(REVOCATION_NOTICE).getByRole('button', { name: 'Dismiss' }));
		expect(screen.queryByText(REVOCATION_NOTICE)).toBeNull();
		expect(screen.getByText(ADMISSION_NOTICE)).toBeTruthy();
	});
});

describe('proposing an admission again', () => {
	// verifies: LLR-md6hhz, LLR-e63yrs
	it('the confirmation is the kept request of that Organisation and invite, among others kept', async () => {
		const other = keptRequest({ invite_id: '3e'.repeat(32), invitee_name: 'Carol Jones', name: 'Carol', surname: 'Jones' });
		const elsewhere = keptRequest({ org_id: 'cc'.repeat(20), invitee_name: 'Dan Doe', name: 'Dan', surname: 'Doe' });
		const first = await openNotices([other, elsewhere, keptRequest()], [admission]);
		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		await settle();
		// Bob's request, not Carol's (another invite) nor Dan's (the same invite id in another Organisation).
		expect(screen.getByText('Invited as').nextElementSibling?.textContent).toBe('Bob Builder');
		expect(screen.getByText('DevicePublicKey').nextElementSibling?.textContent).toBe(DEVICE);
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

		// Kept only in another Organisation: nothing to confirm, so nothing is proposed.
		first.unmount();
		tauri.reset();
		await openNotices([elsewhere], [admission]);
		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		await settle();
		expect(
			screen.getByText('The admission request for this invite is no longer kept, so it cannot be confirmed again.')
		).toBeTruthy();
		expect(screen.queryByRole('button', { name: 'Propose admission again' })).toBeNull();
		expect(tauri.calls('propose_admission')).toEqual([]);
	});

	// verifies: LLR-md6hhz, LLR-e63yrs
	it('the panel proposes an admission again only on the confirmation, and a cancel proposes nothing', async () => {
		await openNotices([keptRequest()], [admission]);

		// Propose again opens the confirmation and proposes nothing yet.
		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([]);
		// The typed invitee name beside the request's name, surname and DevicePublicKey, and the Handle.
		expect(screen.getByText('Invited as').nextElementSibling?.textContent).toBe('Bob Builder');
		expect(screen.getByText('Request').nextElementSibling?.textContent).toBe('Bob Builder');
		expect(screen.getByText('DevicePublicKey').nextElementSibling?.textContent).toBe(DEVICE);
		expect(screen.getByText('Handle').nextElementSibling?.textContent).toBe('bob');
		expect(screen.queryByText('The name in the request is not the name you typed for this Invite.')).toBeNull();

		// Unconfirmed: pressing Propose admission again proposes nothing.
		expect(screen.getByText('Confirm that this request is from the person you invited before admitting.')).toBeTruthy();
		await fireEvent.click(screen.getByRole('button', { name: 'Propose admission again' }));
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([]);

		// Cancel closes the confirmation, proposes nothing, and the notice stays.
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
		await settle();
		expect(screen.queryByRole('button', { name: 'Propose admission again' })).toBeNull();
		expect(tauri.calls('propose_admission')).toEqual([]);
		expect(screen.getByText(ADMISSION_NOTICE)).toBeTruthy();

		// Opened again and confirmed: proposed once, with confirmed true.
		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		expect(screen.getByRole('checkbox', { name: 'This request is from the person I invited' })).toHaveProperty('checked', false);
		await fireEvent.click(screen.getByRole('checkbox', { name: 'This request is from the person I invited' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Propose admission again' }));
		await settle();
		expect(tauri.calls('propose_admission')).toEqual([{ orgId: ORG, inviteId: INVITE, handle: 'bob', confirmed: true }]);
	});

	// verifies: LLR-md6hhz, LLR-e63yrs
	it('the confirmation shows a mismatch again, and no kept request proposes nothing', async () => {
		const elsewhere = { ...admission, invite_id: '3e'.repeat(32), name: 'Dan', surname: 'Doe', handle: 'dan' };
		await openNotices([keptRequest({ invitee_name: 'Robert Builder' })], [admission, elsewhere]);

		await fireEvent.click(notice(ADMISSION_NOTICE).getByRole('button', { name: 'Propose again' }));
		expect(screen.getByRole('alert').textContent).toBe('The name in the request is not the name you typed for this Invite.');
		expect(screen.getByText('Invited as').nextElementSibling?.textContent).toBe('Robert Builder');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

		// The request for that invite is no longer kept: nothing to confirm, so nothing is proposed.
		await fireEvent.click(notice(/The admission of Dan Doe/).getByRole('button', { name: 'Propose again' }));
		await settle();
		expect(
			screen.getByText('The admission request for this invite is no longer kept, so it cannot be confirmed again.')
		).toBeTruthy();
		expect(screen.queryByRole('button', { name: 'Propose admission again' })).toBeNull();
		expect(tauri.calls('propose_admission')).toEqual([]);
	});
});

describe('sendOutcomes', () => {
	// verifies: LLR-fsk4g2
	it("sendOutcomes returns every device's outcome and offers no action", () => {
		const sends = [
			{ device: '3e'.repeat(32), result: 'sent' as const },
			{ device: '4f'.repeat(32), result: 'failed' as const, reason: 'no address' },
			{ device: '5a'.repeat(32), result: 'sent' as const }
		];
		expect(sendOutcomes({ result: 'adopted', org_id: ORG, epoch: 3, own: true, discarded: [], sends })).toEqual(sends);
		expect(sendOutcomes({ result: 'adopted', org_id: ORG, epoch: 3, own: false, discarded: [] })).toEqual([]);
		expect(sendOutcomes({ result: 'in_step', org_id: ORG, epoch: 3 })).toEqual([]);
		expect(Object.keys(sources).length).toBeGreaterThan(5);
		for (const [path, source] of Object.entries(sources)) {
			expect(source, path).not.toMatch(/sendOrganisation|['"]send_organisation['"]/);
		}
	});
});

describe('the page start', () => {
	// verifies: LLR-5mw2dj
	it('a failed listing of the kept admission requests shows under its own label, not the verification one', async () => {
		tauri.refuse('list_admission_requests', 'the store is locked');
		const listing = await openApp();
		expect(screen.getByText('listing the admission requests at start failed: the store is locked')).toBeTruthy();
		expect(screen.queryByText(/verification at start failed/)).toBeNull();
		listing.unmount();

		// The verification at start keeps its own label.
		tauri.reset();
		tauri.refuse('verify_organisation', 'the chain is down');
		await openApp();
		expect(screen.getByText('verification at start failed: the chain is down')).toBeTruthy();
		expect(screen.queryByText(/listing the admission requests at start failed/)).toBeNull();
	});
});
