// @vitest-environment jsdom
/**
 * SDD-xg4nj7: the notices for discarded changes and the send reports, held
 * by the main page and filed from every verification entry: at mount, on
 * "Verify with chain", from an included write and from each
 * `verification-result` event, whichever tab is open (LLR-cgtav9); and the
 * send and request reports, a refusal of the whole included (LLR-9r6y7p).
 * Rendered over the faked Tauri boundary (./fake-tauri.ts).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent, within } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';

const ORG = 'ab'.repeat(20);
const TAIL = `…${ORG.slice(-12)}`;
const DEVICE = '58' + '66'.repeat(31);
const DEVICE_TAIL = `…${DEVICE.slice(-16)}`;

const admission = {
	kind: 'admission',
	invite_id: '1c'.repeat(32),
	name: 'Bob',
	surname: 'Builder',
	handle: 'bob'
};
const revocation = { kind: 'revocation', member_id: '2d'.repeat(32), handle: 'carol' };

/** Another Admin's epoch 3, which left out Bob's admission and Carol's revocation. */
const DISCARDING = { result: 'adopted', org_id: ORG, epoch: 3, own: false, discarded: [admission, revocation] };
/** This node's own epoch 3, sent to two Devices: one sent, one refused. */
const SENT = {
	result: 'adopted',
	org_id: ORG,
	epoch: 3,
	own: true,
	discarded: [],
	sends: [
		{ device: DEVICE, result: 'sent' },
		{ device: DEVICE, result: 'failed', reason: 'refused by the recipient' }
	]
};

const ADMISSION_NOTICE = /The admission of Bob Builder \(Handle bob\) was left out of epoch 3/;
const REVOCATION_NOTICE = /The revocation of Member carol .* was left out of epoch 3/;
const SEND_REPORT = `Sent epoch 3 of ${TAIL} to each Device:`;

/** Opens the Membership panel. */
async function membership() {
	await openTab('S4: Membership');
	await settle();
}

/** The report or notice whose text matches `text`, to act within it as the Admin does. */
function around(text: RegExp | string) {
	return within(screen.getByText(text).parentElement!);
}

/** Every discarded-change notice and the send report of `SENT` are shown, each once. */
function expectAllShownOnce() {
	expect(screen.getAllByText(ADMISSION_NOTICE).length).toBe(1);
	expect(screen.getAllByText(REVOCATION_NOTICE).length).toBe(1);
	expect(screen.getAllByText(SEND_REPORT).length).toBe(1);
	const report = around(SEND_REPORT);
	expect(report.getByText(`${DEVICE_TAIL}: sent`)).toBeTruthy();
	expect(report.getByText(`${DEVICE_TAIL}: failed: refused by the recipient`)).toBeTruthy();
}

beforeEach(() => tauri.reset());

describe('notices and send reports outlive the tab', () => {
	// verifies: LLR-cgtav9
	it('a result announced while another tab is open is shown on the Membership panel', async () => {
		await openApp();
		// The Personas tab is open: the Membership panel is not.
		tauri.emit('verification-result', DISCARDING);
		tauri.emit('verification-result', SENT);
		await settle();
		await membership();
		expectAllShownOnce();
	});

	// verifies: LLR-cgtav9
	it('switching tabs neither clears nor repeats a notice, and only Dismiss removes one', async () => {
		await openApp();
		await membership();
		tauri.emit('verification-result', DISCARDING);
		tauri.emit('verification-result', SENT);
		await settle();
		expectAllShownOnce();
		for (const tab of ['S1: Create Org', 'Personas / Orgs', 'S5: Revoke']) {
			await openTab(tab);
			await membership();
			expectAllShownOnce();
		}
		await fireEvent.click(around(REVOCATION_NOTICE).getByRole('button', { name: 'Dismiss' }));
		await fireEvent.click(around(SEND_REPORT).getByRole('button', { name: 'Dismiss' }));
		await openTab('S3: Admit');
		await membership();
		expect(screen.queryByText(REVOCATION_NOTICE)).toBeNull();
		expect(screen.queryByText(SEND_REPORT)).toBeNull();
		expect(screen.getAllByText(ADMISSION_NOTICE).length).toBe(1);
	});

	// verifies: LLR-cgtav9
	it('the verification at mount files its notices and send reports', async () => {
		tauri.answer('verify_organisation', [DISCARDING, SENT]);
		await openApp();
		await membership();
		expectAllShownOnce();
	});

	// verifies: LLR-cgtav9
	it('"Verify with chain" files its notices and send reports, beside those kept', async () => {
		await openApp();
		await membership();
		expect(screen.queryByText(ADMISSION_NOTICE)).toBeNull();
		tauri.answer('verify_organisation', [DISCARDING, SENT]);
		await openTab('Personas / Orgs');
		await fireEvent.click(screen.getByRole('button', { name: 'Verify with chain' }));
		await settle();
		await membership();
		expectAllShownOnce();
		// A later verification that finds the Organisation in step removes none.
		tauri.answer('verify_organisation', [{ result: 'in_step', org_id: ORG, epoch: 3 }]);
		await openTab('Personas / Orgs');
		await fireEvent.click(screen.getByRole('button', { name: 'Verify with chain' }));
		await settle();
		await membership();
		expectAllShownOnce();
	});

	// verifies: LLR-cgtav9
	it("an included write's verification files its send report; a refused write files none", async () => {
		tauri.answer('list_orgs', [
			{ org_id: ORG, epoch: 1, root_hash: '9a'.repeat(32), member_count: 1, verification: { status: 'not_checked' } }
		]);
		tauri.answer('propose_updated_organisation', { epoch: 2, root: '9b'.repeat(32), org_public_key: '9c'.repeat(32) });
		tauri.answer('write_organisation_to_chain', { result: 'refused', reason: 'not yet' });
		await openApp();
		await membership();
		await fireEvent.click(screen.getByRole('button', { name: 'Propose next epoch' }));
		await settle();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(screen.queryByText(/Sent epoch 2/)).toBeNull();
		tauri.answer('write_organisation_to_chain', {
			result: 'included',
			epoch: 2,
			verification: { result: 'adopted', org_id: ORG, epoch: 2, own: true, discarded: [], sends: [] }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		await openTab('S2: Invite');
		await membership();
		expect(screen.getByText(`Sent epoch 2 of ${TAIL} to each Device:`)).toBeTruthy();
		expect(screen.getByText('No other Device to send to.')).toBeTruthy();
	});
});

describe('send and request reports', () => {
	// verifies: LLR-9r6y7p
	it("a send refused as a whole is shown with org-io's reason, not as an empty list", async () => {
		await openApp();
		tauri.emit('verification-result', {
			result: 'adopted',
			org_id: ORG,
			epoch: 3,
			own: true,
			discarded: [],
			sends: [],
			sends_refused: 'Organisation abab is not held here'
		});
		await settle();
		await membership();
		expect(
			screen.getByText(`The send of epoch 3 of ${TAIL} was refused: Organisation abab is not held here`)
		).toBeTruthy();
		expect(screen.queryByText(SEND_REPORT)).toBeNull();
		expect(screen.queryByText('No other Device to send to.')).toBeNull();
	});

	// verifies: LLR-9r6y7p
	it("a request refused as a whole is shown with org-io's reason and offers no action but Dismiss", async () => {
		await openApp();
		tauri.emit('verification-result', {
			result: 'behind',
			org_id: ORG,
			chain_epoch: 4,
			requests: [],
			requests_refused: 'no endpoint is bound'
		});
		await settle();
		await membership();
		const text = `The request of ${TAIL} from its Members (chain at epoch 4) was refused: no endpoint is bound`;
		const report = around(text);
		expect(report.getAllByRole('button').map((button) => button.textContent)).toEqual(['Dismiss']);
		await fireEvent.click(report.getByRole('button', { name: 'Dismiss' }));
		expect(screen.queryByText(text)).toBeNull();
	});

	// verifies: LLR-9r6y7p
	it('each Device of a send is shown, a later report beside the earlier, and an entry with no outcome adds none', async () => {
		await openApp();
		await membership();
		tauri.emit('verification-result', SENT);
		tauri.emit('verification-result', { ...SENT, epoch: 4, sends: [{ device: DEVICE, result: 'sent' }] });
		// Neither another Admin's adoption nor a behind result with no refusal is a send report.
		tauri.emit('verification-result', { result: 'adopted', org_id: ORG, epoch: 5, own: false, discarded: [] });
		await settle();
		expect(screen.getAllByText(SEND_REPORT).length).toBe(1);
		expect(screen.getByText(`Sent epoch 4 of ${TAIL} to each Device:`)).toBeTruthy();
		expect(screen.queryByText(/epoch 5/)).toBeNull();
		const first = around(SEND_REPORT);
		expect(first.getByText(`${DEVICE_TAIL}: sent`)).toBeTruthy();
		expect(first.getByText(`${DEVICE_TAIL}: failed: refused by the recipient`)).toBeTruthy();
		expect(first.getAllByRole('button').map((button) => button.textContent)).toEqual(['Dismiss']);
	});
});
