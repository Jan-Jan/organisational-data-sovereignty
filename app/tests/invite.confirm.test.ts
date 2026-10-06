/**
 * REQ-ab2mfz (RC-wzb48r), LLR-n2u4uf: before an Invite reply is produced the
 * user is told that nothing verified the Invite's sender or the Organisation
 * name it states, and that the reply reveals the chosen Persona's handle, name
 * and surname to its sender; the reply is produced only once the user
 * confirms. Tested on the extracted decision, never a component (as
 * revoke.validate).
 */
import { describe, it, expect } from 'vitest';
import { REPLY_WARNING, replyGate } from '../src/lib/invite';

describe('the Invite reply warning and gate', () => {
	// verifies: LLR-n2u4uf
	it('states each thing the user must be told', () => {
		expect(REPLY_WARNING).toMatch(/nothing has verified who sent this invite/i);
		expect(REPLY_WARNING).toMatch(/organisation name/i);
		expect(REPLY_WARNING).toMatch(/handle, name and surname/i);
	});

	// verifies: LLR-n2u4uf
	it('refuses until the user has confirmed', () => {
		expect(replyGate({ personaId: 'p-1', confirmed: false })).toEqual({
			ok: false,
			message: 'Read the warning and confirm before replying.'
		});
	});

	// verifies: LLR-n2u4uf
	it('refuses without a chosen Persona even when confirmed', () => {
		expect(replyGate({ personaId: '  ', confirmed: true }).ok).toBe(false);
	});

	// verifies: LLR-n2u4uf
	it('allows the reply once a Persona is chosen and the user confirmed', () => {
		expect(replyGate({ personaId: 'p-1', confirmed: true })).toEqual({ ok: true });
	});
});
