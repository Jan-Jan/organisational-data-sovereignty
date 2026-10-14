/**
 * LLR-hwcj9n: what the webview shows of a reconcile, and when it asks for a
 * reconcile (SDD-huk773). The discard wrappers and view, and their tests,
 * were removed on 2026-10-13 (owner ruling of that day, plan task T15d).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn(async (): Promise<unknown> => []) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import { reconcileOrganisations, type ReconciliationDto } from '../src/lib/api';
import { reconciliationLine, verificationLabel } from '../src/lib/reconcile';
import pageSource from '../src/routes/+page.svelte?raw';
import listSource from '../src/lib/components/PersonaList.svelte?raw';

const ORG = 'ab'.repeat(20);

describe('reconciliationLine', () => {
	// verifies: LLR-hwcj9n
	it('names the Organisation and the result, with the epoch where there is one', () => {
		expect(reconciliationLine({ result: 'in_step', org_id: ORG, epoch: 3 })).toBe(`${ORG}: in step at epoch 3`);
		expect(reconciliationLine({ result: 'committed', org_id: ORG, epoch: 4, recipients: ['cd'.repeat(32)] })).toBe(
			`${ORG}: committed at epoch 4 (1 recipient Device)`
		);
		expect(reconciliationLine({ result: 'behind', org_id: ORG, chain_epoch: 9 })).toBe(
			`${ORG}: behind the chain (chain at epoch 9)`
		);
		expect(reconciliationLine({ result: 'removed', org_id: ORG })).toBe(`${ORG}: this device was removed`);
	});

	// verifies: LLR-hwcj9n
	it('shows the reason for a refused or unverified result and never calls it in step', () => {
		const refused = reconciliationLine({ result: 'refused', org_id: ORG, reason: 'stale' });
		expect(refused).toBe(`${ORG}: refused: stale`);
		const unverified = reconciliationLine({ result: 'unverified', org_id: ORG, reason: 'chain not configured' });
		expect(unverified).toBe(`${ORG}: unverified: chain not configured`);
		expect(unverified).not.toContain('in step');
	});

	// verifies: LLR-hwcj9n
	it('renders a result it does not know as unverified rather than in step', () => {
		const unknown = { result: 'something_new', org_id: ORG } as unknown as ReconciliationDto;
		expect(reconciliationLine(unknown)).toBe(`${ORG}: unverified: unknown result something_new`);
	});
});

describe('verificationLabel', () => {
	// verifies: LLR-hwcj9n
	it('renders each status, an unverified one with its reason', () => {
		expect(verificationLabel({ status: 'verified' })).toBe('verified');
		expect(verificationLabel({ status: 'unverified', reason: 'read failed' })).toBe('unverified: read failed');
		expect(verificationLabel({ status: 'not_checked' })).toBe('not checked');
	});
});

describe('command wrappers', () => {
	beforeEach(() => invoke.mockClear());

	// verifies: LLR-hwcj9n
	it('reconcileOrganisations invokes reconcile_organisations with no argument', async () => {
		await reconcileOrganisations();
		expect(invoke).toHaveBeenCalledWith('reconcile_organisations');
		expect(reconcileOrganisations.length).toBe(0);
	});

});

describe('the main page', () => {
	// verifies: LLR-hwcj9n
	it('asks for a reconcile once when it mounts', () => {
		const mount = pageSource.match(/onMount\(\(\) => \{([\s\S]*?)\n\t\}\);/);
		expect(mount, 'the page has an onMount block').not.toBeNull();
		expect(mount![1].match(/reconcileOrganisations\(/g)?.length).toBe(1);
	});
});

describe('the Organisation list', () => {
	// Review round 1, finding-6: the list asks for a reconcile again when the
	// user presses its reconcile button, and shows that reconcile's results.
	// verifies: LLR-hwcj9n
	it('calls reconcile_organisations when its reconcile button is pressed', () => {
		const button = listSource.match(/<button onclick=\{(\w+)\}[^>]*>[^<]*Reconcile with chain[^<]*<\/button>/);
		expect(button, 'the list has a reconcile button with a click handler').not.toBeNull();
		const handler = listSource.match(new RegExp(`async function ${button![1]}\\(\\) \\{([\\s\\S]*?)\\n\\t\\}`));
		expect(handler, `the list defines the handler ${button![1]}`).not.toBeNull();
		expect(handler![1]).toMatch(/requested = await reconcileOrganisations\(\);/);
		expect(listSource).toMatch(/let shown = \$derived\(requested \?\? reconciled\);/);
	});
});
