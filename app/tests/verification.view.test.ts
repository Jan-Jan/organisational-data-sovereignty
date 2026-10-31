// @vitest-environment jsdom
/**
 * SDD-swt9fs: what the webview shows of a verification (LLR-mfzr6z,
 * LLR-njb6mr, LLR-zkpsw5, LLR-84cxpt), and SDD-ua2d9b: when it asks for one
 * (LLR-j2x9jb). Tested on the extracted decisions, and on the rendered page
 * over the faked Tauri boundary (./fake-tauri.ts).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { render, screen, fireEvent } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp, openTab, settle } from './page';
import Page from '../src/routes/+page.svelte';
import {
	verifyOrganisations,
	writeOrganisationToChain,
	type VerificationEntryDto
} from '../src/lib/api';
import { verificationLine, verificationLabel } from '../src/lib/verification';

/** The lines under "Last verifications", top to bottom. */
function shownLines(): string[] {
	const heading = screen.getByRole('heading', { name: 'Last verifications' });
	const list = heading.nextElementSibling as HTMLElement;
	return Array.from(list.querySelectorAll('li')).map((item) => item.textContent ?? '');
}

/** The founders the founding-writes list offers, top to bottom, each beside its Write to chain button. */
function offeredFoundings(): string[] {
	return screen
		.queryAllByRole('button', { name: 'Write to chain' })
		.map((button) => (button.parentElement?.textContent ?? '').replace('Write to chain', '').replace('founding by', '').trim());
}

beforeEach(() => tauri.reset());

const sources = import.meta.glob('../src/**/*.{ts,svelte}', { query: '?raw', import: 'default', eager: true }) as Record<
	string,
	string
>;

const ORG = 'ab'.repeat(20);
const DEVICE = 'cd'.repeat(32);

describe('verificationLine', () => {
	// verifies: LLR-mfzr6z
	it('names the Organisation and the result, with the epoch where there is one', () => {
		expect(verificationLine({ result: 'in_step', org_id: ORG, epoch: 3 })).toBe(`${ORG}: in step at epoch 3`);
		expect(verificationLine({ result: 'adopted', org_id: ORG, epoch: 4, own: true, discarded: [], sends: [] })).toBe(
			`${ORG}: adopted epoch 4`
		);
		expect(verificationLine({ result: 'behind', org_id: ORG, chain_epoch: 9 })).toBe(
			`${ORG}: behind the chain (chain at epoch 9)`
		);
		expect(verificationLine({ result: 'removed', org_id: ORG })).toBe(`${ORG}: this device was removed`);
	});

	// verifies: LLR-mfzr6z
	it('shows the reason for a refused or unverified result and never calls it in step', () => {
		const inconsistent = verificationLine({ result: 'inconsistent', org_id: ORG, reason: 'root differs' });
		expect(inconsistent).toBe(`${ORG}: inconsistent: root differs`);
		const unverified = verificationLine({ result: 'unverified', org_id: ORG, reason: 'chain not configured' });
		expect(unverified).toBe(`${ORG}: unverified: chain not configured`);
		for (const line of [inconsistent, unverified]) {
			expect(line).not.toContain('in step');
			expect(line).not.toContain('adopted');
		}
	});

	// verifies: LLR-mfzr6z
	it('renders a result it does not know as unverified rather than in step', () => {
		const unknown = { result: 'something_new', org_id: ORG } as unknown as VerificationEntryDto;
		expect(verificationLine(unknown)).toBe(`${ORG}: unverified: unknown result something_new`);
	});

	// verifies: LLR-mfzr6z
	it('verificationLine renders every result and never a removed or behind one as in step', () => {
		const results: VerificationEntryDto[] = [
			{ result: 'in_step', org_id: ORG, epoch: 1 },
			{ result: 'adopted', org_id: ORG, epoch: 2, own: false, discarded: [] },
			{ result: 'removed', org_id: ORG },
			{ result: 'behind', org_id: ORG, chain_epoch: 5, requests: [] },
			{ result: 'inconsistent', org_id: ORG, reason: 'r1' },
			{ result: 'unverified', org_id: ORG, reason: 'r2' }
		];
		const lines = results.map(verificationLine);
		expect(lines).toEqual([
			`${ORG}: in step at epoch 1`,
			`${ORG}: adopted epoch 2`,
			`${ORG}: this device was removed`,
			`${ORG}: behind the chain (chain at epoch 5)`,
			`${ORG}: inconsistent: r1`,
			`${ORG}: unverified: r2`
		]);
		for (const line of lines.slice(2)) {
			expect(line).not.toMatch(/in step|adopted/);
		}
	});

	// verifies: LLR-mfzr6z
	it('follows a settled result with its write and names a founding with no proxy by its founder', () => {
		expect(
			verificationLine({ result: 'adopted', org_id: ORG, epoch: 4, own: true, discarded: [], write: 'adopted' })
		).toBe(`${ORG}: adopted epoch 4; write adopted`);
		expect(verificationLine({ result: 'in_step', org_id: ORG, epoch: 3, write: 'not_executed' })).toBe(
			`${ORG}: in step at epoch 3; write not executed`
		);
		expect(verificationLine({ result: 'behind', org_id: ORG, chain_epoch: 9, write: 'deprecated' })).toBe(
			`${ORG}: behind the chain (chain at epoch 9); write deprecated`
		);
		const unknownWrite = verificationLine({ result: 'unverified', org_id: ORG, reason: 'down', write: 'still_unknown' });
		expect(unknownWrite).toBe(`${ORG}: unverified: down; write still unknown`);
		// The label never changes how the result is worded: an unverified result stays unverified.
		expect(unknownWrite.startsWith(verificationLine({ result: 'unverified', org_id: ORG, reason: 'down' }))).toBe(true);
		expect(
			verificationLine({ result: 'unverified', org_id: null, founder: DEVICE, reason: 'the founding is not written' })
		).toBe(`founding by ${DEVICE}: unverified: the founding is not written`);
	});
});

describe('verificationLabel', () => {
	// verifies: LLR-njb6mr
	it('renders each status, an unverified one with its reason', () => {
		expect(verificationLabel({ status: 'unverified', reason: 'read failed' })).toBe('unverified: read failed');
		expect(verificationLabel({ status: 'not_checked' })).toBe('not checked');
	});

	// verifies: LLR-njb6mr
	it('verificationLabel renders each status', () => {
		expect(verificationLabel({ status: 'in_step', epoch: 3 })).toBe('in step');
		expect(verificationLabel({ status: 'adopted', epoch: 4 })).toBe('adopted');
		// `removed` (LLR-g8wfy9, and LLR-njb6mr's label list) is
		// shown as removed, never as not checked or in step.
		expect(verificationLabel({ status: 'removed' })).toBe('removed');
		expect(verificationLabel({ status: 'behind', chain_epoch: 9 })).toBe('behind (chain at epoch 9)');
		expect(verificationLabel({ status: 'inconsistent', reason: 'root differs' })).toBe('inconsistent: root differs');
		expect(verificationLabel({ status: 'unverified', reason: 'down' })).toBe('unverified: down');
		expect(verificationLabel({ status: 'not_checked' })).toBe('not checked');
	});

	// verifies: LLR-njb6mr
	it('a status it does not know is labelled unverified, never not checked', () => {
		const unknown = { status: 'revoked_elsewhere' } as unknown as Parameters<typeof verificationLabel>[0];
		expect(verificationLabel(unknown)).toBe('unverified: unknown status revoked_elsewhere');
	});
});

describe('command wrappers', () => {
	// verifies: LLR-j2x9jb
	it('verifyOrganisations invokes verify_organisation with no argument', async () => {
		await verifyOrganisations();
		expect(tauri.invoke).toHaveBeenCalledWith('verify_organisation');
		expect(verifyOrganisations.length).toBe(0);
	});
});

describe('the main page', () => {
	// verifies: LLR-j2x9jb
	it('asks for a verification once when it mounts', async () => {
		// No timer of its own: the periodic verification is the Rust half's (LLR-3ampqm),
		// so an hour with the page open asks for none.
		vi.useFakeTimers();
		try {
			render(Page);
			await vi.advanceTimersByTimeAsync(0);
			expect(tauri.calls('verify_organisation')).toEqual([undefined]);
			await vi.advanceTimersByTimeAsync(60 * 60 * 1000);
			expect(tauri.calls('verify_organisation')).toEqual([undefined]);
		} finally {
			vi.useRealTimers();
		}
	});
});

describe('the Organisation list', () => {
	// verifies: LLR-j2x9jb
	it('calls verify_organisation when its Verify with chain button is pressed', async () => {
		await openApp();
		expect(tauri.calls('verify_organisation')).toEqual([undefined]);
		tauri.answer('verify_organisation', [{ result: 'in_step', org_id: ORG, epoch: 3 }]);
		await fireEvent.click(screen.getByRole('button', { name: 'Verify with chain' }));
		await settle();
		expect(tauri.calls('verify_organisation')).toEqual([undefined, undefined]);
		// The verification the user asked for is shown in the list.
		expect(shownLines()).toEqual([`${ORG}: in step at epoch 3`]);
	});
});

describe('no request action', () => {
	// verifies: LLR-zkpsw5
	it('no source defines or invokes a request action', () => {
		expect(Object.keys(sources).length).toBeGreaterThan(5);
		for (const [path, source] of Object.entries(sources)) {
			expect(source, path).not.toMatch(/offersRequest|requestOrganisation|['"]request_organisation['"]/);
		}
		// A behind line is the same whether or not its requests report names a failed Device.
		const plain = verificationLine({ result: 'behind', org_id: ORG, chain_epoch: 9 });
		const failed = verificationLine({
			result: 'behind',
			org_id: ORG,
			chain_epoch: 9,
			requests: [{ device: DEVICE, result: 'failed', reason: 'timed out' }]
		});
		const sent = verificationLine({
			result: 'behind',
			org_id: ORG,
			chain_epoch: 9,
			requests: [{ device: DEVICE, result: 'sent' }]
		});
		expect(failed).toBe(plain);
		expect(sent).toBe(plain);
	});
});

describe('verification-result events', () => {
	// verifies: LLR-84cxpt
	it('a verification-result event is filed newest first through verificationLine', async () => {
		const older: VerificationEntryDto = { result: 'in_step', org_id: ORG, epoch: 1 };
		const event: VerificationEntryDto = { result: 'adopted', org_id: ORG, epoch: 2, own: true, discarded: [], sends: [] };

		// The start's entry, then the event, in one list, newest first.
		tauri.answer('verify_organisation', [older]);
		await openApp();
		expect(shownLines()).toEqual([`${ORG}: in step at epoch 1`]);
		tauri.emit('verification-result', event);
		await settle();
		expect(shownLines()).toEqual([`${ORG}: adopted epoch 2`, `${ORG}: in step at epoch 1`]);
	});
});

describe('verification-result events at the start', () => {
	// verifies: LLR-84cxpt
	// Abnormal timing. A result
	// announced while the start's own verification is still running (the
	// periodic run or the receiver loop got there first) is filed, not lost
	// or overwritten when the start's entries arrive after it, and the one
	// filed last is on top.
	it('an event that arrives before the start verification returns is kept beside its entries', async () => {
		const announced: VerificationEntryDto = { result: 'adopted', org_id: ORG, epoch: 2, own: true, discarded: [], sends: [] };
		const listed: VerificationEntryDto = { result: 'in_step', org_id: ORG, epoch: 2 };
		tauri.answer('verify_organisation', () => {
			tauri.emit('verification-result', announced);
			return [listed];
		});
		await openApp();
		expect(shownLines()).toEqual([`${ORG}: in step at epoch 2`, `${ORG}: adopted epoch 2`]);
	});
});

describe('the founding write', () => {

	// verifies: LLR-rhc555
	it('offers each held founding once, by the founder its proposal or its verification entry carries', async () => {
		const PROPOSED = 'ef'.repeat(32);
		// A restarted app: its first verification names a founding after its proxy is recorded.
		tauri.answer('verify_organisation', [{ result: 'unverified', org_id: ORG, founder: DEVICE, reason: 'the founding is not written' }]);
		const restarted = await openApp();
		expect(offeredFoundings()).toEqual([DEVICE]);
		restarted.unmount();

		// An Organisation's entry carries no founder and is never offered.
		tauri.reset();
		tauri.answer('verify_organisation', [{ result: 'in_step', org_id: ORG, epoch: 3 }]);
		const organisation = await openApp();
		expect(offeredFoundings()).toEqual([]);
		organisation.unmount();

		// A founding just proposed is offered first, then those the verification names,
		// and a founder both proposed and named is offered once.
		for (const founder of [PROPOSED, DEVICE]) {
			tauri.reset();
			tauri.answer('verify_organisation', [{ result: 'unverified', org_id: null, founder: DEVICE, reason: 'no proxy yet' }]);
			tauri.answer('propose_person', { device_public_key: DEVICE, name: 'Alice', surname: 'Smith', status: 'provisional' });
			tauri.answer('propose_founding_organisation', { persona_id: 'ab'.repeat(32), status: 'provisional_founding', founder });
			const page = await openApp();
			await openTab('S1: Create Org');
			await fireEvent.input(screen.getByLabelText('Handle'), { target: { value: 'alice' } });
			await fireEvent.input(screen.getByLabelText('Name'), { target: { value: 'Alice' } });
			await fireEvent.input(screen.getByLabelText('Surname'), { target: { value: 'Smith' } });
			await fireEvent.input(screen.getByLabelText('Organisation name'), { target: { value: 'Acme' } });
			await fireEvent.click(screen.getByRole('button', { name: 'Propose Person + founding' }));
			await settle();
			await openTab('Personas / Orgs');
			await settle();
			expect(offeredFoundings()).toEqual(founder === DEVICE ? [DEVICE] : [PROPOSED, DEVICE]);
			page.unmount();
		}
	});

	// verifies: LLR-rhc555
	it('writes a founding with writeOrganisationToChain from its Write to chain button', async () => {
		tauri.answer('write_organisation_to_chain', { result: 'refused', reason: 'not yet' });
		await writeOrganisationToChain({ founder: DEVICE });
		expect(tauri.invoke).toHaveBeenCalledWith('write_organisation_to_chain', { founder: DEVICE });

		// On the page: the list offers the founding its first verification names, by its founder; the
		// button writes by that key, and an included write's verification is filed into the same list.
		tauri.reset();
		tauri.answer('verify_organisation', [{ result: 'unverified', org_id: null, founder: DEVICE, reason: 'no proxy yet' }]);
		tauri.answer('write_organisation_to_chain', {
			result: 'included',
			epoch: 0,
			// The real included founding write's verification adopts it (backend.json).
			verification: { result: 'adopted', org_id: ORG, founder: DEVICE, epoch: 0, own: true, discarded: [], sends: [] }
		});
		await openApp();
		const write = screen.getByRole('button', { name: 'Write to chain' });
		expect(write.parentElement?.textContent).toMatch(new RegExp(`^\\s*founding by ${DEVICE}\\s+Write to chain\\s*$`));
		await fireEvent.click(write);
		await settle();
		expect(tauri.calls('write_organisation_to_chain')).toEqual([{ founder: DEVICE }]);
		expect(shownLines()).toEqual([`${ORG}: adopted epoch 0`, `founding by ${DEVICE}: unverified: no proxy yet`]);
		// Written: no longer offered for writing.
		expect(screen.queryByRole('button', { name: 'Write to chain' })).toBeNull();
	});
});
