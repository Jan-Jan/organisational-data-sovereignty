/**
 * LLR-rt8gdz: one Persona, one Organisation. The Personas offered to reply to
 * an Invite as are only those bound to no Organisation. Tested on the
 * extracted decision, never a component (as invite.confirm).
 */
import { describe, it, expect } from 'vitest';
import type { PersonaDto } from '../src/lib/api';
import { unboundPersonas } from '../src/lib/invite';

function persona(persona_id: string, org_id: string | null): PersonaDto {
	return { persona_id, org_id, handle: persona_id, name: 'N', surname: 'S', status: 'Active' };
}

describe('the Personas offered to reply as', () => {
	// verifies: LLR-rt8gdz
	it('keeps only the Personas bound to no Organisation, in order', () => {
		const list = [
			persona('a', null),
			persona('b', 'aa'.repeat(20)),
			persona('c', null),
			persona('d', 'bb'.repeat(20))
		];
		expect(unboundPersonas(list).map((p) => p.persona_id)).toEqual(['a', 'c']);
	});

	// verifies: LLR-rt8gdz
	it('offers none when every Persona is bound', () => {
		expect(unboundPersonas([persona('b', 'aa'.repeat(20)), persona('d', 'bb'.repeat(20))])).toEqual([]);
	});

	// verifies: LLR-rt8gdz
	it('offers none from an empty list', () => {
		expect(unboundPersonas([])).toEqual([]);
	});
});
