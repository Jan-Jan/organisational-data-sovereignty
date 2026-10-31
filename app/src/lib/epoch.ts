/**
 * SDD-p7sdy3: the Admin's epoch commands in the webview, as pure
 * decisions: which proposed updates are offered for writing, and what the
 * Admin is told of a proposal and of a write. The components render what
 * these return.
 */
import type { ProposedOrganisationDto, WriteDto } from './api';

/** LLR-tbtrb3: an updated Organisation proposed and not yet written. */
export interface ProposedUpdate {
	org_id: string;
	epoch: number;
}

/** How an Organisation is named in the Admin's messages. */
function orgName(orgId: string): string {
	return `…${orgId.slice(-12)}`;
}

/** LLR-tbtrb3: the proposed updates with `orgId`'s latest proposal, replacing an earlier one of it in its place. */
export function fileProposal(held: ProposedUpdate[], orgId: string, proposed: ProposedOrganisationDto): ProposedUpdate[] {
	const update = { org_id: orgId, epoch: proposed.epoch };
	const at = held.findIndex((kept) => kept.org_id === orgId);
	if (at === -1) return [...held, update];
	return held.map((kept, index) => (index === at ? update : kept));
}

/** LLR-tbtrb3: an included write withdraws `orgId`'s offer; any other outcome keeps it. */
export function afterWrite(held: ProposedUpdate[], orgId: string, written: WriteDto): ProposedUpdate[] {
	return written.result === 'included' ? held.filter((kept) => kept.org_id !== orgId) : held;
}

/** LLR-5vgpkd: what the Admin is told of a proposal, or of its refusal with org-io's text. */
export function proposalMessage(
	orgId: string,
	outcome: { proposed: ProposedOrganisationDto } | { refused: string }
): string {
	if ('refused' in outcome) return `Nothing proposed for ${orgName(orgId)}: ${outcome.refused}`;
	return `Epoch ${outcome.proposed.epoch} proposed for ${orgName(orgId)}. It is not on the chain until you press "Write to chain".`;
}

/** LLR-tbtrb3: what the Admin is told of a write's outcome, or of a rejected invoke with its text. */
export function writeMessage(outcome: { written: WriteDto } | { rejected: string }): string {
	if ('rejected' in outcome) return `Write to chain failed: ${outcome.rejected}`;
	const written = outcome.written;
	switch (written.result) {
		case 'included':
			return `Epoch ${written.epoch} written to the chain.`;
		case 'refused':
			return `Write refused: ${written.reason}`;
		case 'unknown':
			return `Write outcome unknown: ${written.reason}`;
	}
}

/** LLR-b59vcx: what the Admit and Revoke panels say once a change is proposed. */
export const NEXT_EPOCH_NOTE =
	'It is written to the chain only with the next epoch: press "Propose next epoch", then "Write to chain", on S4: Membership.';
