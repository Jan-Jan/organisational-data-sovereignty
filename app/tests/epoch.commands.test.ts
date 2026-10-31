// @vitest-environment jsdom
/**
 * SDD-p7sdy3: the Admin's epoch commands in the webview. Proposing the next
 * epoch, writing a proposed update to the chain, cancelling an Invite, and
 * what the Admit and Revoke panels say a proposal did (LLR-dt8ak9,
 * LLR-5vgpkd, LLR-tbtrb3, LLR-e7bbm5, LLR-b59vcx). Rendered over the faked
 * Tauri boundary (./fake-tauri.ts), whose answers backend.json pins.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent, within } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import { cancelInvite, proposeUpdatedOrganisation } from '../src/lib/api';

const ORG = 'ab'.repeat(20);
const TAIL = `…${ORG.slice(-12)}`;
const INVITE = '1c'.repeat(32);
const MEMBER = '2d'.repeat(32);

const HELD = [
	{
		org_id: ORG,
		epoch: 1,
		root_hash: '9a'.repeat(32),
		member_count: 1,
		verification: { status: 'not_checked' }
	}
];
const PROPOSED = { epoch: 2, root: '9b'.repeat(32), org_public_key: '9c'.repeat(32) };
const INCLUDED = {
	result: 'included',
	epoch: 2,
	verification: { result: 'adopted', org_id: ORG, epoch: 2, own: true, discarded: [], sends: [] }
};
const KEPT = {
	org_id: ORG,
	invite_id: INVITE,
	invitee_name: 'Bob Builder',
	name: 'Bob',
	surname: 'Builder',
	preferred_handles: ['bob', 'builder'],
	device_public_key: 'cd'.repeat(32)
};

const PROPOSED_TEXT = `Epoch 2 proposed for ${TAIL}. It is not on the chain until you press "Write to chain".`;
const NEXT_EPOCH = 'It is written to the chain only with the next epoch: press "Propose next epoch", then "Write to chain", on S4: Membership.';

/** Opens the app holding one Organisation, on the Membership panel. */
async function openMembership() {
	tauri.answer('list_orgs', HELD);
	await openApp();
	await openTab('S4: Membership');
	await settle();
}

/** Presses "Propose next epoch" for the one Organisation held. */
async function proposeNextEpoch() {
	await fireEvent.click(screen.getByRole('button', { name: 'Propose next epoch' }));
	await settle();
}

beforeEach(() => tauri.reset());

describe('the wrappers', () => {
	// verifies: LLR-dt8ak9
	it('proposeUpdatedOrganisation and cancelInvite pass their arguments alone and reject with the refusal', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		tauri.answer('cancel_invite', null);
		await expect(proposeUpdatedOrganisation(ORG)).resolves.toEqual(PROPOSED);
		await expect(cancelInvite(ORG, INVITE)).resolves.toBeNull();
		expect(tauri.calls('propose_updated_organisation')).toEqual([{ orgId: ORG }]);
		expect(tauri.calls('cancel_invite')).toEqual([{ orgId: ORG, inviteId: INVITE }]);
		expect(proposeUpdatedOrganisation.length).toBe(1);
		expect(cancelInvite.length).toBe(2);

		// A refusal rejects with org-io's text unchanged.
		tauri.refuse('propose_updated_organisation', 'no provisional change is held');
		tauri.refuse('cancel_invite', 'no admission request is kept for this invite');
		await expect(proposeUpdatedOrganisation(ORG)).rejects.toBe('no provisional change is held');
		await expect(cancelInvite(ORG, INVITE)).rejects.toBe('no admission request is kept for this invite');
	});
});

describe('Propose next epoch', () => {
	// verifies: LLR-5vgpkd
	it('proposes once for the Organisation and says it is not yet on the chain', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		await openMembership();
		await proposeNextEpoch();
		expect(tauri.calls('propose_updated_organisation')).toEqual([{ orgId: ORG }]);
		expect(screen.getByText(PROPOSED_TEXT)).toBeTruthy();
		// Proposing writes nothing.
		expect(tauri.calls('write_organisation_to_chain')).toEqual([]);
		expect(screen.getByRole('button', { name: 'Write to chain' })).toBeTruthy();
	});

	// verifies: LLR-5vgpkd
	it('a second press while the first proposal is in flight proposes nothing more, the button disabled', async () => {
		const proposal = tauri.defer('propose_updated_organisation');
		await openMembership();
		await proposeNextEpoch();
		const button = screen.getByRole('button', { name: 'Propose next epoch' }) as HTMLButtonElement;
		expect(button.disabled).toBe(true);
		await proposeNextEpoch();
		expect(tauri.calls('propose_updated_organisation')).toEqual([{ orgId: ORG }]);
		proposal.answer(PROPOSED);
		await settle();
		expect(screen.getByText(PROPOSED_TEXT)).toBeTruthy();
		expect(button.disabled).toBe(false);
		expect(tauri.calls('propose_updated_organisation')).toEqual([{ orgId: ORG }]);
	});

	// verifies: LLR-5vgpkd
	it("a refusal, nothing to propose included, is shown as org-io's text and offers no write", async () => {
		tauri.refuse('propose_updated_organisation', 'no provisional change is held');
		await openMembership();
		await proposeNextEpoch();
		expect(tauri.calls('propose_updated_organisation')).toEqual([{ orgId: ORG }]);
		expect(screen.getByText(`Nothing proposed for ${TAIL}: no provisional change is held`)).toBeTruthy();
		expect(screen.queryByText(/proposed for .*It is not on the chain/)).toBeNull();
		expect(screen.queryByRole('button', { name: 'Write to chain' })).toBeNull();
	});

	// verifies: LLR-5vgpkd
	it('no Organisation held offers no proposal', async () => {
		await openApp();
		await openTab('S4: Membership');
		await settle();
		expect(screen.queryByRole('button', { name: 'Propose next epoch' })).toBeNull();
		expect(screen.getByText('No Organisation is held, so there is no epoch to propose.')).toBeTruthy();
	});
});

describe('Write to chain for a proposed update', () => {
	// verifies: LLR-tbtrb3
	it('writes by the Organisation once, shows the epoch written and withdraws the offer, across a tab switch', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		tauri.answer('write_organisation_to_chain', INCLUDED);
		await openMembership();
		await proposeNextEpoch();
		// The offer outlives a tab switch.
		await openTab('S3: Admit');
		await openTab('S4: Membership');
		await settle();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(tauri.calls('write_organisation_to_chain')).toEqual([{ orgId: ORG }]);
		expect(screen.getByText('Epoch 2 written to the chain.')).toBeTruthy();
		expect(screen.queryByRole('button', { name: 'Write to chain' })).toBeNull();
		// Its verification is filed as every verification is.
		await openTab('Personas / Orgs');
		await settle();
		expect(screen.getByText(`${ORG}: adopted epoch 2`)).toBeTruthy();
	});

	// verifies: LLR-tbtrb3
	it('a refused write shows the reason and keeps the offer', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		tauri.answer('write_organisation_to_chain', { result: 'refused', reason: 'not yet' });
		await openMembership();
		await proposeNextEpoch();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(screen.getByText('Write refused: not yet')).toBeTruthy();
		expect(screen.queryByText(/written to the chain\./)).toBeNull();
		expect(screen.getByRole('button', { name: 'Write to chain' })).toBeTruthy();
	});

	// verifies: LLR-tbtrb3
	it('a write of unknown outcome says so, never that the epoch is written, and keeps the offer', async () => {
		const reason = 'the write timed out after 90 s and may yet execute on chain';
		tauri.answer('propose_updated_organisation', PROPOSED);
		tauri.answer('write_organisation_to_chain', { result: 'unknown', reason });
		await openMembership();
		await proposeNextEpoch();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(screen.getByText(`Write outcome unknown: ${reason}`)).toBeTruthy();
		expect(screen.queryByText(/written/i)).toBeNull();
		expect(screen.getByText(`epoch 2 proposed for ${TAIL}`)).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Write to chain' })).toBeTruthy();
	});

	// verifies: LLR-tbtrb3
	it('a write still in flight across a tab switch stays guarded, and its outcome is shown on return', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		const write = tauri.defer('write_organisation_to_chain');
		await openMembership();
		await proposeNextEpoch();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		await openTab('S3: Admit');
		await openTab('S4: Membership');
		await settle();
		// The remounted panel offers no second write while the first is in flight.
		expect((screen.getByRole('button', { name: 'Write to chain' }) as HTMLButtonElement).disabled).toBe(true);
		await openTab('S3: Admit');
		write.answer({ result: 'refused', reason: 'not yet' });
		await settle();
		await openTab('S4: Membership');
		await settle();
		expect(screen.getByText('Write refused: not yet')).toBeTruthy();
		expect((screen.getByRole('button', { name: 'Write to chain' }) as HTMLButtonElement).disabled).toBe(false);
		expect(tauri.calls('write_organisation_to_chain')).toEqual([{ orgId: ORG }]);
	});

	// verifies: LLR-5vgpkd
	it('a proposal still in flight across a tab switch stays guarded, and its outcome is shown on return', async () => {
		const proposal = tauri.defer('propose_updated_organisation');
		await openMembership();
		await proposeNextEpoch();
		await openTab('S3: Admit');
		await openTab('S4: Membership');
		await settle();
		// The remounted panel offers no second proposal while the first is in flight.
		expect((screen.getByRole('button', { name: 'Propose next epoch' }) as HTMLButtonElement).disabled).toBe(true);
		await openTab('S3: Admit');
		proposal.refuse('no provisional change is held');
		await settle();
		await openTab('S4: Membership');
		await settle();
		expect(screen.getByText(`Nothing proposed for ${TAIL}: no provisional change is held`)).toBeTruthy();
		expect((screen.getByRole('button', { name: 'Propose next epoch' }) as HTMLButtonElement).disabled).toBe(false);
	});

	// verifies: LLR-tbtrb3
	it('a rejected write shows its text and keeps the offer', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		tauri.refuse('write_organisation_to_chain', 'the chain is not configured');
		await openMembership();
		await proposeNextEpoch();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(screen.getByText('Write to chain failed: the chain is not configured')).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Write to chain' })).toBeTruthy();
	});

	// verifies: LLR-tbtrb3
	it('a second proposal of the same Organisation replaces the first offer', async () => {
		tauri.answer('propose_updated_organisation', PROPOSED);
		await openMembership();
		await proposeNextEpoch();
		await proposeNextEpoch();
		expect(screen.getAllByRole('button', { name: 'Write to chain' }).length).toBe(1);
	});

	// verifies: LLR-tbtrb3
	it("a second proposal at a later epoch replaces the first: the offer shows the second's epoch, not the first's", async () => {
		const answered = [PROPOSED, { ...PROPOSED, epoch: 3 }];
		tauri.answer('propose_updated_organisation', () => answered.shift());
		await openMembership();
		await proposeNextEpoch();
		expect(screen.getByText(`epoch 2 proposed for ${TAIL}`)).toBeTruthy();
		await proposeNextEpoch();
		expect(screen.getByText(`epoch 3 proposed for ${TAIL}`)).toBeTruthy();
		expect(screen.queryByText(`epoch 2 proposed for ${TAIL}`)).toBeNull();
		expect(screen.getAllByRole('button', { name: 'Write to chain' }).length).toBe(1);
	});
});

describe('cancelling an Invite', () => {
	// verifies: LLR-e7bbm5
	it("cancels the kept request's invite once and removes the request from every panel", async () => {
		tauri.answer('list_admission_requests', [KEPT]);
		tauri.answer('cancel_invite', null);
		await openApp();
		await openTab('S3: Admit');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel invite' }));
		await settle();
		expect(tauri.calls('cancel_invite')).toEqual([{ orgId: ORG, inviteId: INVITE }]);
		expect(screen.getByText('Invite cancelled: the admission request of Bob Builder is removed.')).toBeTruthy();
		expect(screen.queryByRole('button', { name: /Bob Builder — invite/ })).toBeNull();
		expect(screen.getByText('No admission requests kept.')).toBeTruthy();
		// Gone from the page's list, not only from this panel.
		await openTab('S4: Membership');
		await openTab('S3: Admit');
		expect(screen.queryByRole('button', { name: /Bob Builder — invite/ })).toBeNull();
	});

	// verifies: LLR-e7bbm5
	it('a cancel still in flight across a tab switch stays guarded, and its outcome is shown on return', async () => {
		tauri.answer('list_admission_requests', [KEPT]);
		const cancel = tauri.defer('cancel_invite');
		await openApp();
		await openTab('S3: Admit');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel invite' }));
		await settle();
		await openTab('S4: Membership');
		await openTab('S3: Admit');
		await settle();
		expect((screen.getByRole('button', { name: 'Cancel invite' }) as HTMLButtonElement).disabled).toBe(true);
		await openTab('S4: Membership');
		cancel.refuse('no admission request is kept for this invite');
		await settle();
		await openTab('S3: Admit');
		await settle();
		expect(screen.getByText('no admission request is kept for this invite')).toBeTruthy();
		expect((screen.getByRole('button', { name: 'Cancel invite' }) as HTMLButtonElement).disabled).toBe(false);
		expect(tauri.calls('cancel_invite')).toEqual([{ orgId: ORG, inviteId: INVITE }]);
	});

	// verifies: LLR-e7bbm5
	it("a refused cancel shows org-io's text and the request stays", async () => {
		tauri.answer('list_admission_requests', [KEPT]);
		tauri.refuse('cancel_invite', 'no admission request is kept for this invite');
		await openApp();
		await openTab('S3: Admit');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel invite' }));
		await settle();
		expect(screen.getByText('no admission request is kept for this invite')).toBeTruthy();
		expect(screen.queryByText(/Invite cancelled/)).toBeNull();
		expect(screen.getByRole('button', { name: /Bob Builder — invite/ })).toBeTruthy();
	});
});

describe('what a proposal is said to have done', () => {
	// verifies: LLR-b59vcx
	it('the Admit panel says the admission is proposed and written only with the next epoch', async () => {
		tauri.answer('list_admission_requests', [KEPT]);
		tauri.answer('propose_admission', null);
		await openApp();
		await openTab('S3: Admit');
		await fireEvent.click(screen.getByRole('button', { name: /Bob Builder — invite/ }));
		await fireEvent.click(screen.getByRole('checkbox', { name: 'This request is from the person I invited' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Propose admission' }));
		await settle();
		expect(screen.getByText(`Admission proposed. ${NEXT_EPOCH}`)).toBeTruthy();
		expect(screen.queryByText(/submitted|waiting for/i)).toBeNull();
	});

	// verifies: LLR-b59vcx
	it('the Revoke panel says the revocation is proposed and written only with the next epoch, never submitted', async () => {
		tauri.answer('list_orgs', HELD);
		tauri.answer('propose_revocation', null);
		await openApp();
		await openTab('S5: Revoke');
		await settle();
		await fireEvent.input(screen.getByRole('textbox', { name: /Member ID/ }), { target: { value: MEMBER } });
		await fireEvent.click(screen.getByRole('button', { name: 'Propose revocation' }));
		await settle();
		expect(tauri.calls('propose_revocation')).toEqual([{ orgId: ORG, memberId: MEMBER }]);
		expect(screen.getByText(`Revocation proposed. ${NEXT_EPOCH}`)).toBeTruthy();
		expect(screen.queryByText(/submitted|Waiting for the remote peer/i)).toBeNull();
	});

	// verifies: LLR-b59vcx
	it('a refused proposal says nothing was proposed, and the one that follows says it is proposed', async () => {
		tauri.answer('list_orgs', HELD);
		tauri.refuse('propose_revocation', 'the Member is not held');
		await openApp();
		await openTab('S5: Revoke');
		await settle();
		await fireEvent.input(screen.getByRole('textbox', { name: /Member ID/ }), { target: { value: MEMBER } });
		await fireEvent.click(screen.getByRole('button', { name: 'Propose revocation' }));
		await settle();
		expect(screen.getByText('the Member is not held')).toBeTruthy();
		expect(screen.queryByText(/Revocation proposed/)).toBeNull();
		expect(within(document.body).queryByText(/submitted/i)).toBeNull();
		// Accepted on a second press: now, and only now, it is proposed.
		tauri.answer('propose_revocation', null);
		await fireEvent.click(screen.getByRole('button', { name: 'Propose revocation' }));
		await settle();
		expect(screen.getByText(`Revocation proposed. ${NEXT_EPOCH}`)).toBeTruthy();
	});
});
