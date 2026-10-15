/**
 * LLR-pms47e: one Persona, one Organisation, on the webview's side. The
 * Personas offered to reply to an Invite as are only those bound to no
 * Organisation. Tested on the extracted decision, never a component (as
 * invite.confirm).
 *
 * *2026-10-14 (S4, task T15):* the first three tests verified the
 * one-Persona-one-Organisation item that moved to org-io with the backend
 * refusals (S4 task T13); its only ID left them and they keep running with no
 * `verifies:` line (the T9 precedent). The webview's half is LLR-pms47e, the
 * app's, which the fourth test cites.
 *
 * *Review round 1, 2026-10-14:* the first three now cite LLR-pms47e as
 * well, inherited from the moved item: they test the same filter, and
 * LLR-pms47e lists them.
 *
 * *Review round 2, 2026-10-15 (finding-14):* the moved item is named in
 * prose as org-io's one-Persona-one-Organisation item (moved to org-io in
 * T13), and stays uncited: it is org-io's, and an app test cites only the
 * app's items.
 */
import { describe, it, expect } from 'vitest';
import type { PersonaDto } from '../src/lib/api';
import { unboundPersonas } from '../src/lib/invite';

function persona(persona_id: string, org_id: string | null): PersonaDto {
	return { persona_id, org_id, handle: persona_id, name: 'N', surname: 'S', status: 'Active' };
}

describe('the Personas offered to reply as', () => {
	// inherited from org-io's one-Persona-one-Organisation item (moved to org-io in T13), review round 1, 2026-10-14
	// verifies: LLR-pms47e
	it('keeps only the Personas bound to no Organisation, in order', () => {
		const list = [
			persona('a', null),
			persona('b', 'aa'.repeat(20)),
			persona('c', null),
			persona('d', 'bb'.repeat(20))
		];
		expect(unboundPersonas(list).map((p) => p.persona_id)).toEqual(['a', 'c']);
	});

	// inherited from org-io's one-Persona-one-Organisation item (moved to org-io in T13), review round 1, 2026-10-14
	// verifies: LLR-pms47e
	it('offers none when every Persona is bound', () => {
		expect(unboundPersonas([persona('b', 'aa'.repeat(20)), persona('d', 'bb'.repeat(20))])).toEqual([]);
	});

	// inherited from org-io's one-Persona-one-Organisation item (moved to org-io in T13), review round 1, 2026-10-14
	// verifies: LLR-pms47e
	it('offers none from an empty list', () => {
		expect(unboundPersonas([])).toEqual([]);
	});

	// verifies: LLR-pms47e
	it('offers a Persona whose Organisation is the empty string as bound', () => {
		// Boundary value: only a null `org_id` is unbound. An empty string is
		// not the backend's "no Organisation", so it is not offered.
		const list = [persona('a', ''), persona('b', null)];
		expect(unboundPersonas(list).map((p) => p.persona_id)).toEqual(['b']);
	});
});
