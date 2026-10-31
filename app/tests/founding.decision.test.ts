/**
 * SDD-g5wmxt: the creation form's decision on a refused Person proposal,
 * tested at the item's own interface (TEST_GUIDELINES Seams 2 criterion 4):
 * a pure function the creation form calls, apart from the IPC calls.
 */

import { describe, it, expect } from 'vitest';
import { personRefusalStep } from '../src/lib/founding';
import foundingSource from '../src/lib/founding.ts?raw';

describe('personRefusalStep', () => {
	// verifies: LLR-d8u93q
	it('proceeds past a Person held already, whatever its wording', () => {
		expect(personRefusalStep({ kind: 'person_already_held', message: 'a Person is held already' })).toEqual({
			proceed: true
		});
		expect(personRefusalStep({ kind: 'person_already_held', message: '' })).toEqual({ proceed: true });
	});

	// verifies: LLR-d8u93q
	it('stops on a refusal of any other kind, with its message, though worded like a held Person', () => {
		expect(personRefusalStep({ kind: 'refused', message: 'a Person is held already? name: must not be empty' })).toEqual({
			proceed: false,
			message: 'a Person is held already? name: must not be empty'
		});
	});

	// verifies: LLR-d8u93q
	it('stops on a rejection that is a plain string, with that string', () => {
		expect(personRefusalStep('IPC: command propose_person not found')).toEqual({
			proceed: false,
			message: 'IPC: command propose_person not found'
		});
	});

	// verifies: LLR-d8u93q
	it('stops on any other rejection with its text, never undefined', () => {
		expect(personRefusalStep(new Error('channel closed'))).toEqual({ proceed: false, message: 'Error: channel closed' });
		expect(personRefusalStep({ kind: 'person_already_held' })).toEqual({
			proceed: false,
			message: '[object Object]'
		});
		expect(personRefusalStep(undefined)).toEqual({ proceed: false, message: 'undefined' });
		expect(personRefusalStep(null)).toEqual({ proceed: false, message: 'null' });
	});
});

describe('founding.ts imports', () => {
	// verifies: LLR-5sxa6c
	it('imports types alone, so it names no IPC adapter function', () => {
		const source: string = foundingSource;
		const imports = source.match(/^\s*import\b[^;]*;/gm) ?? [];
		for (const statement of imports) {
			expect(statement).toMatch(/^\s*import\s+type\s/);
		}
		expect(source).not.toMatch(/@tauri-apps\/api/);
	});
});
