import type { PersonDto, PersonRefusalDto, ProposedFoundingDto } from './api';

/** What the creation form proposed: the Person (`null` when one was held already) and the founding. */
export interface FoundingProposals {
	person: PersonDto | null;
	founding: ProposedFoundingDto;
}

/** LLR-d8u93q: whether the founding is still proposed after `propose_person` refused, or what is shown. */
export type PersonRefusalStep = { proceed: true } | { proceed: false; message: string };

/** A `PersonRefusalDto`, as `propose_person` rejects with (LLR-k57uqv). */
function isPersonRefusal(refusal: unknown): refusal is PersonRefusalDto {
	return (
		typeof refusal === 'object' &&
		refusal !== null &&
		typeof (refusal as PersonRefusalDto).kind === 'string' &&
		typeof (refusal as PersonRefusalDto).message === 'string'
	);
}

/**
 * SDD-g5wmxt, LLR-d8u93q: the creation form's decision on a rejected
 * `propose_person`. org-io holds one Person per device and refuses a second
 * (kind `person_already_held`); that refusal alone is passed over, because
 * the Person it names is the one the form needs. It is decided by the kind,
 * never by org-io's wording. Any other refusal stops the founding with its
 * message; any other rejection (a plain string, as a failed IPC call rejects
 * with, or any other value) stops it with that value's text.
 */
export function personRefusalStep(refusal: unknown): PersonRefusalStep {
	if (isPersonRefusal(refusal)) {
		return refusal.kind === 'person_already_held' ? { proceed: true } : { proceed: false, message: refusal.message };
	}
	return { proceed: false, message: typeof refusal === 'string' ? refusal : String(refusal) };
}
