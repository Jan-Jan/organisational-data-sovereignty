import type { VerificationEntryDto, VerificationDto, WriteLabelDto } from './api';

/** LLR-mfzr6z: the words for a settled write. */
const WRITE_WORDS: Record<WriteLabelDto, string> = {
	adopted: 'write adopted',
	not_executed: 'write not executed',
	deprecated: 'write deprecated',
	still_unknown: 'write still unknown'
};

/** The Organisation an entry names, or, for a founding with no proxy yet, its founder (LLR-jf63e4). */
function subject(entry: { org_id: string | null; founder?: string }): string {
	return entry.org_id ?? `founding by ${entry.founder}`;
}

/** The result's own words, before any write label. */
function resultLine(entry: VerificationEntryDto): string {
	const name = subject(entry);
	switch (entry.result) {
		case 'in_step':
			return `${name}: in step at epoch ${entry.epoch}`;
		case 'adopted':
			return `${name}: adopted epoch ${entry.epoch}`;
		case 'removed':
			return `${name}: this device was removed`;
		case 'behind':
			return `${name}: behind the chain (chain at epoch ${entry.chain_epoch})`;
		case 'inconsistent':
			return `${name}: inconsistent: ${entry.reason}`;
		case 'unverified':
			return `${name}: unverified: ${entry.reason}`;
		default: {
			const unknown = entry as { result: string };
			return `${name}: unverified: unknown result ${unknown.result}`;
		}
	}
}

/**
 * LLR-mfzr6z: one verification result as one line. The result is org-io's;
 * this only words it. A removed, behind, inconsistent or unverified result
 * is never worded as in step or adopted, and a result this view does not
 * know is shown as unverified (HAZ-9fmhm4). A behind line does not depend
 * on its `requests` report (LLR-zkpsw5). A result carrying a `write` label
 * is its result's line followed by the write's settlement; the label never
 * changes how the result itself is worded.
 */
export function verificationLine(entry: VerificationEntryDto): string {
	const line = resultLine(entry);
	return entry.write ? `${line}; ${WRITE_WORDS[entry.write]}` : line;
}

/** LLR-njb6mr: an Organisation's last verification as the list shows it. */
export function verificationLabel(verification: VerificationDto): string {
	switch (verification.status) {
		case 'in_step':
			return 'in step';
		case 'adopted':
			return 'adopted';
		case 'removed':
			return 'removed';
		case 'behind':
			return `behind (chain at epoch ${verification.chain_epoch})`;
		case 'inconsistent':
			return `inconsistent: ${verification.reason}`;
		case 'unverified':
			return `unverified: ${verification.reason}`;
		case 'not_checked':
			return 'not checked';
		default: {
			// A status this view does not know is shown as unverified, as resultLine does (HAZ-9fmhm4).
			const unknown = verification as { status: string };
			return `unverified: unknown status ${unknown.status}`;
		}
	}
}

/**
 * LLR-84cxpt: the list of results the view shows, newest first. The
 * entries `verifyOrganisations` returned and a `verification-result`
 * event's entry are filed the same way.
 */
export function fileVerificationResults(
	filed: VerificationEntryDto[],
	entries: VerificationEntryDto[]
): VerificationEntryDto[] {
	return [...entries, ...filed];
}

/**
 * LLR-rhc555: the founders whose founding the view
 * offers to write to the chain, each once: those `proposed` (the `founder`
 * of `proposeFoundingOrganisation`'s result, LLR-6ff735) first, then those
 * the filed results' founding entries carry (LLR-jf63e4), newest first, so
 * a restarted app finds every held founding in its first verification. A
 * founder whose newest entry is `adopted` or `in_step` is written and is
 * not offered.
 */
export function foundingWrites(results: VerificationEntryDto[], proposed: string[]): string[] {
	const founders = [...proposed, ...results.flatMap((entry) => (entry.founder ? [entry.founder] : []))];
	return [...new Set(founders)].filter((founder) => {
		const newest = results.find((entry) => entry.founder === founder);
		return newest === undefined || (newest.result !== 'adopted' && newest.result !== 'in_step');
	});
}
