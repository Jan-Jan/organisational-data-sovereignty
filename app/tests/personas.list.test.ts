// @vitest-environment jsdom
/**
 * SDD-23vjd8: what the Persona list shows as a Persona's Handle
 * (LLR-q735zx). Tested on the decision itself (TEST_GUIDELINES Seams 2
 * criterion 4) and on the rendered page over the faked Tauri boundary
 * (./fake-tauri.ts), which answers `list_personas` only with shapes
 * app/tests/contract/backend.json holds.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));

import { screen, within } from '@testing-library/svelte';
import { tauri } from './fake-tauri';
import { openApp } from './page';
import { handleLabel, NO_HANDLE_YET } from '../src/lib/personas';

beforeEach(() => tauri.reset());

const ORG = 'ab'.repeat(20);

/** A Persona as `list_personas` answers it: a provisional Member of ORG has no Handle; a founder, before its proxy, has its Handle and no Organisation. */
function persona(id: string, handle: string | null, name: string) {
	return { persona_id: id.repeat(64), org_id: handle === null ? ORG : null, handle, name, surname: 'Builder', status: 'Proposed' };
}

/** The text of `row`'s cell under the "Handle" column heading. */
function handleCell(row: HTMLElement): string {
	const headings = screen.getAllByRole('columnheader').map((heading) => heading.textContent);
	return within(row).getAllByRole('cell')[headings.indexOf('Handle')].textContent ?? '';
}

describe('handleLabel', () => {
	// verifies: LLR-q735zx
	it('shows a Handle as it is', () => {
		expect(handleLabel('bob')).toBe('bob');
	});

	// verifies: LLR-q735zx
	it('shows a missing Handle as the stated label, never as null', () => {
		expect(handleLabel(null)).toBe(NO_HANDLE_YET);
		expect(NO_HANDLE_YET).not.toMatch(/null|undefined/);
		expect(NO_HANDLE_YET.trim()).not.toBe('');
	});
});

describe('the Persona list', () => {
	// verifies: LLR-q735zx
	it('shows a provisional Member with no Handle by the stated label and not as "null"', async () => {
		tauri.answer('list_personas', [persona('7', null, 'Bob')]);
		await openApp();
		const row = (await screen.findByText('Bob Builder')).closest('tr') as HTMLElement;
		expect(handleCell(row)).toBe(NO_HANDLE_YET);
		expect(row.textContent).not.toContain('null');
	});

	// verifies: LLR-q735zx
	it('shows a Persona that holds a Handle by that Handle', async () => {
		tauri.answer('list_personas', [persona('8', 'alice', 'Alice')]);
		await openApp();
		const row = (await screen.findByText('Alice Builder')).closest('tr') as HTMLElement;
		expect(handleCell(row)).toBe('alice');
	});
});
