/**
 * SDD-7aq69w, REQ-2yh3c8 (LLR-qcg6bz, LLR-prh433): what a provisional Member is told
 * before its first admission request, and the decision that gates sending
 * it. There is no Persona to choose: each Member proposal is its own new
 * Persona (org-io's Persona per proposal).
 */

export const ADMISSION_REQUEST_WARNING =
	'Nothing has verified who sent this Invite, or the Organisation name it states. ' +
	'Your admission request will reveal the name, surname and preferred Handles you entered to the Device the Invite names.';

export type RequestCheck = { ok: true } | { ok: false; message: string };

/**
 * LLR-qcg6bz: refuses, naming the missing confirmation, until the user
 * confirms the first request. LLR-prh433: a request for a provisional
 * Member whose earlier request the user confirmed is allowed with no
 * confirmation asked again.
 */
export function admissionRequestGate(input: { confirmedBefore: boolean; confirmed: boolean }): RequestCheck {
	if (input.confirmedBefore || input.confirmed) {
		return { ok: true };
	}
	return { ok: false, message: 'Read the warning and confirm before requesting admission.' };
}
