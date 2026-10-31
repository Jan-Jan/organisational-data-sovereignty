// @vitest-environment jsdom
/**
 * The creation form (CreateOrg.svelte): the Person proposed if none
 * is held, then the founding, whose `founder` the form shows and hands to
 * the founding-writes list (LLR-k57uqv, LLR-6ff735, LLR-rhc555). Tested on
 * the rendered page over the faked Tauri boundary (./fake-tauri.ts). The
 * refusal of a second Person is passed over by its kind
 * (`person_already_held`), not by org-io's wording.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, fireEvent } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';

const DEVICE = 'cd'.repeat(32);
const FOUNDER = 'ef'.repeat(32);
const ORG = 'ab'.repeat(20);
const PERSON = { device_public_key: DEVICE, name: 'Alice', surname: 'Smith', status: 'provisional' };
const FOUNDING = { persona_id: 'ab'.repeat(32), status: 'provisional_founding', founder: FOUNDER };
const PERSON_HELD = { kind: 'person_already_held', message: `a Person is held already (${DEVICE}); nothing created` };

/** Fills the creation form with `handle` and presses its button. */
async function proposeFromTheForm(handle = 'alice') {
	await openApp();
	await openTab('S1: Create Org');
	await fireEvent.input(screen.getByLabelText('Handle'), { target: { value: handle } });
	await fireEvent.input(screen.getByLabelText('Name'), { target: { value: 'Alice' } });
	await fireEvent.input(screen.getByLabelText('Surname'), { target: { value: 'Smith' } });
	await fireEvent.input(screen.getByLabelText('Organisation name'), { target: { value: 'Acme' } });
	await fireEvent.click(screen.getByRole('button', { name: 'Propose Person + founding' }));
	await settle();
}

beforeEach(() => tauri.reset());

describe('the creation form', () => {
	// verifies: LLR-k57uqv, LLR-6ff735
	it('proposes the Person and then the founding, and shows both', async () => {
		tauri.answer('propose_person', PERSON);
		tauri.answer('propose_founding_organisation', FOUNDING);
		await proposeFromTheForm();
		expect(tauri.calls('propose_person')).toEqual([{ name: 'Alice', surname: 'Smith' }]);
		expect(tauri.calls('propose_founding_organisation')).toEqual([
			{ name: 'Alice', surname: 'Smith', handle: 'alice', orgName: 'Acme' }
		]);
		expect(screen.getByText('Person proposed:').textContent).toBe(`Person proposed: ${DEVICE}`);
		expect(screen.getByText("Founder's DevicePublicKey:").textContent).toBe(`Founder's DevicePublicKey: ${FOUNDER}`);
	});

	// verifies: LLR-k57uqv, LLR-6ff735
	it('a Person held already is passed over by its kind and the founding is still proposed', async () => {
		tauri.refuse('propose_person', PERSON_HELD);
		tauri.answer('propose_founding_organisation', FOUNDING);
		await proposeFromTheForm();
		expect(tauri.commands().filter((name) => name.startsWith('propose_'))).toEqual([
			'propose_person',
			'propose_founding_organisation'
		]);
		expect(screen.queryByText('Person proposed:')).toBeNull();
		expect(screen.getByText("Founder's DevicePublicKey:").textContent).toBe(`Founder's DevicePublicKey: ${FOUNDER}`);
	});

	// verifies: LLR-k57uqv
	it('any other refusal of the Person is shown as its text and no founding is proposed', async () => {
		// Worded like the held Person, but of the other kind: the kind decides, not the text.
		tauri.refuse('propose_person', { kind: 'refused', message: 'a Person is held already? name: must not be empty' });
		tauri.answer('propose_founding_organisation', FOUNDING);
		await proposeFromTheForm();
		expect(screen.getByText('a Person is held already? name: must not be empty')).toBeTruthy();
		expect(tauri.calls('propose_founding_organisation')).toEqual([]);
	});

	// verifies: LLR-6ff735
	it('a refused founding is shown as org-io words it', async () => {
		tauri.answer('propose_person', PERSON);
		tauri.refuse('propose_founding_organisation', 'handle: must not be empty');
		await proposeFromTheForm('');
		expect(screen.getByText('handle: must not be empty')).toBeTruthy();
		expect(screen.queryByText("Founder's DevicePublicKey:")).toBeNull();
	});

	// verifies: LLR-rhc555
	it('the creation form shows the founder and hands it to the founding-writes list', async () => {
		tauri.answer('propose_person', PERSON);
		tauri.answer('propose_founding_organisation', FOUNDING);
		tauri.answer('write_organisation_to_chain', {
			result: 'included',
			epoch: 0,
			verification: { result: 'adopted', org_id: ORG, founder: FOUNDER, epoch: 0, own: true, discarded: [], sends: [] }
		});
		await proposeFromTheForm();

		// The form shows the founder's DevicePublicKey and writes nothing itself.
		expect(screen.getByText("Founder's DevicePublicKey:").textContent).toBe(`Founder's DevicePublicKey: ${FOUNDER}`);
		expect(tauri.commands()).not.toContainEqual(expect.stringMatching(/create_persona|create_organisation|write_organisation_to_chain/));

		// The founding-writes list offers it by that key, and its button writes it by that key.
		await openTab('Personas / Orgs');
		await settle();
		expect(screen.getByText(`founding by ${FOUNDER}`, { exact: false })).toBeTruthy();
		await fireEvent.click(screen.getByRole('button', { name: 'Write to chain' }));
		await settle();
		expect(tauri.calls('write_organisation_to_chain')).toEqual([{ founder: FOUNDER }]);
	});
});
