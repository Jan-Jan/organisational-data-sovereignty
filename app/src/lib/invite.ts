/**
 * REQ-ab2mfz / RC-wzb48r (LLR-n2u4uf): what the user is told before an
 * Invite reply is produced, and the decision that gates producing it.
 *
 * The reply's keys are per-Organisation and expose nothing about the person,
 * so the warning names only what the reply does reveal: the handle, name and
 * surname entered for the chosen Persona.
 */
import type { PersonaDto } from './api';

export const REPLY_WARNING =
	'Nothing has verified who sent this Invite, or the Organisation name it states. ' +
	'Your reply will reveal the handle, name and surname entered for the Persona you choose to whoever sent it.';

/**
 * LLR-rt8gdz: the Personas to offer as the one to reply as — only those bound
 * to no Organisation (one Persona, one Organisation). The backend refuses a
 * bound one as well.
 */
export function unboundPersonas(personas: PersonaDto[]): PersonaDto[] {
	return personas.filter((p) => p.org_id === null);
}

export type ReplyCheck = { ok: true } | { ok: false; message: string };

export function replyGate(input: { personaId: string; confirmed: boolean }): ReplyCheck {
	if (input.personaId.trim() === '') {
		return { ok: false, message: 'Choose the Persona to reply as.' };
	}
	if (!input.confirmed) {
		return { ok: false, message: 'Read the warning and confirm before replying.' };
	}
	return { ok: true };
}
