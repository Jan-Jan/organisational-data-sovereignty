// @vitest-environment jsdom
/**
 * SDD-g9ugu7: the removed Devices view's words (LLR-cs9bmw) and when it
 * reads the list (LLR-x9qbhu), over the faked Tauri boundary.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));
import { render, screen, fireEvent } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import backend from './contract/backend.json';
import RemovedDevices from '../src/lib/components/RemovedDevices.svelte';
import { removedDeviceLine } from '../src/lib/removed';

const ORG = 'ab'.repeat(20);
const [ACKNOWLEDGED] = backend.answers.list_removed_devices.one_acknowledged;
const [UNACKNOWLEDGED] = backend.answers.list_removed_devices.one_unacknowledged;
/** A `verification-result` payload backend.json holds, for the Organisation `org_id`. */
const verified = (org_id: string) => ({ ...backend.events['verification-result'].adopted_own_update, org_id });

/** LLR-cs9bmw: no text the panel shows contains a deletion word but the statement's. */
const expectNoDeletionWords = () =>
	expect((document.body.textContent ?? '').replace('not proof that its data was deleted', '')).not.toMatch(/deleted|erased|wiped/);

beforeEach(() => tauri.reset());

describe('removed Devices view', () => {
	// verifies: LLR-cs9bmw
	it('words an acknowledgement as the Device statement, never as deletion', async () => {
		expect(removedDeviceLine(ACKNOWLEDGED)).toBe('acknowledged its removal at epoch 4');
		expect(removedDeviceLine(UNACKNOWLEDGED)).toBe('no acknowledgement received by this Device');
		tauri.answer('list_removed_devices', [ACKNOWLEDGED]);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('Removed Devices (Admin)')).toBeTruthy();
		expect(
			screen.getByText(
				"An acknowledgement is the removed Device's own signed statement that it acted on its removal. It is not proof that its data was deleted."
			)
		).toBeTruthy();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		expect(screen.getByText(ACKNOWLEDGED.member_id, { exact: false })).toBeTruthy();
		expect(screen.getByText(ACKNOWLEDGED.device_key, { exact: false })).toBeTruthy();
		expectNoDeletionWords();
	});

	// verifies: LLR-cs9bmw
	it('says so when no Device is removed, shows a refusal as its text, and neither shows a deletion word', async () => {
		tauri.answer('list_removed_devices', []);
		const first = render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('No removed Devices.')).toBeTruthy();
		expectNoDeletionWords();
		first.unmount();
		tauri.refuse('list_removed_devices', 'organisation not held');
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('organisation not held')).toBeTruthy();
		expectNoDeletionWords();
	});

	// verifies: LLR-x9qbhu
	it('reads the list when shown, on Refresh, and after a verification result for its Organisation only', async () => {
		tauri.answer('list_removed_devices', [UNACKNOWLEDGED]);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(tauri.calls('list_removed_devices')).toEqual([{ orgId: ORG }]);
		expect(screen.getByText('no acknowledgement received by this Device')).toBeTruthy();
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await settle();
		expect(tauri.calls('list_removed_devices')).toHaveLength(2);
		tauri.emit('verification-result', verified('cd'.repeat(20)));
		await settle();
		expect(tauri.calls('list_removed_devices')).toHaveLength(2);
		tauri.emit('admission-requested', { ...backend.events['admission-requested'].kept, org_id: ORG });
		await settle();
		expect(tauri.calls('list_removed_devices')).toHaveLength(2);
		tauri.answer('list_removed_devices', [ACKNOWLEDGED]);
		tauri.emit('verification-result', verified(ORG));
		await settle();
		expect(tauri.calls('list_removed_devices')).toHaveLength(3);
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		expect(screen.queryByText('no acknowledgement received by this Device')).toBeNull();
		expect(screen.queryByText(UNACKNOWLEDGED.device_key, { exact: false })).toBeNull();
	});

	// Abnormal: with no subscription the panel cannot read again after a
	// verification result, so it says so rather than leave the rejection unhandled.
	// verifies: LLR-x9qbhu
	it('a refused subscription is shown on the panel, and the rows still are', async () => {
		tauri.refuseListening('the event bus is closed');
		tauri.answer('list_removed_devices', [ACKNOWLEDGED]);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('listening for verification results failed: the event bus is closed')).toBeTruthy();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
	});

	// verifies: LLR-cs9bmw
	it('while the first answer is in flight it says nothing of the list, then shows it', async () => {
		const inFlight = tauri.defer('list_removed_devices');
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('Removed Devices (Admin)')).toBeTruthy();
		expect(screen.queryByText('No removed Devices.')).toBeNull();
		inFlight.answer([]);
		await settle();
		expect(screen.getByText('No removed Devices.')).toBeTruthy();
	});

	// Abnormal: two reads overlap and their answers land out of order.
	// verifies: LLR-x9qbhu
	it('an earlier read answered after a later one is dropped, so the latest answer stays shown', async () => {
		tauri.answer('list_removed_devices', []);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		const earlier = tauri.defer('list_removed_devices');
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		const later = tauri.defer('list_removed_devices');
		tauri.emit('verification-result', verified(ORG));
		await settle();
		expect(tauri.calls('list_removed_devices')).toHaveLength(3);
		later.answer([ACKNOWLEDGED]);
		await settle();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		earlier.answer([UNACKNOWLEDGED]);
		await settle();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		expect(screen.queryByText('no acknowledgement received by this Device')).toBeNull();
	});

	// Abnormal: a refusal of an earlier read landing last is dropped too.
	// verifies: LLR-x9qbhu
	it('an earlier read refused after a later one was answered leaves the latest rows shown', async () => {
		tauri.answer('list_removed_devices', []);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		const earlier = tauri.defer('list_removed_devices');
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		const later = tauri.defer('list_removed_devices');
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await settle();
		later.answer([ACKNOWLEDGED]);
		await settle();
		earlier.refuse('organisation not held');
		await settle();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		expect(screen.queryByText('organisation not held')).toBeNull();
	});

	// verifies: LLR-cs9bmw, LLR-x9qbhu
	it('a refused Refresh replaces the rows with its text, and the next answer replaces the text with rows', async () => {
		tauri.answer('list_removed_devices', [ACKNOWLEDGED]);
		render(RemovedDevices, { orgId: ORG });
		await settle();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		tauri.refuse('list_removed_devices', 'organisation not held');
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await settle();
		expect(screen.getByText('organisation not held')).toBeTruthy();
		expect(screen.queryByText('acknowledged its removal at epoch 4')).toBeNull();
		tauri.answer('list_removed_devices', [UNACKNOWLEDGED]);
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await settle();
		expect(screen.queryByText('organisation not held')).toBeNull();
		expect(screen.getByText('no acknowledgement received by this Device')).toBeTruthy();
	});
});

describe('the removed Devices panel on the Revoke tab', () => {
	const OTHER = 'cd'.repeat(20);
	const [HELD] = backend.answers.list_orgs.one_not_checked;

	// verifies: LLR-x9qbhu
	it('shows the selected Organisation\'s removed Devices, and reads them again for another selected', async () => {
		tauri.answer('list_orgs', [HELD, { ...HELD, org_id: OTHER }]);
		tauri.answer('list_removed_devices', (args: { orgId: string }) =>
			args.orgId === HELD.org_id ? [ACKNOWLEDGED] : [UNACKNOWLEDGED]
		);
		await openApp();
		await openTab('S5: Revoke');
		await settle();
		expect(tauri.calls('list_removed_devices')).toEqual([{ orgId: HELD.org_id }]);
		expect(screen.getByText('Removed Devices (Admin)')).toBeTruthy();
		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
		await fireEvent.change(screen.getByRole('combobox'), { target: { value: OTHER } });
		await settle();
		expect(tauri.calls('list_removed_devices')).toEqual([{ orgId: HELD.org_id }, { orgId: OTHER }]);
		expect(screen.getByText('no acknowledgement received by this Device')).toBeTruthy();
		expect(screen.queryByText('acknowledged its removal at epoch 4')).toBeNull();
	});
});
