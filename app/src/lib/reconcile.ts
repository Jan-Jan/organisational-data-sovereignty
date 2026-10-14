import type { ReconciliationDto, VerificationDto } from './api';

/**
 * LLR-hwcj9n: one reconcile result as one line. The result is org-io's; this
 * only words it. A result this view does not know is shown as unverified,
 * never as in step (HAZ-9fmhm4).
 */
export function reconciliationLine(result: ReconciliationDto): string {
	switch (result.result) {
		case 'in_step':
			return `${result.org_id}: in step at epoch ${result.epoch}`;
		case 'committed': {
			const count = result.recipients.length;
			return `${result.org_id}: committed at epoch ${result.epoch} (${count} recipient Device${count === 1 ? '' : 's'})`;
		}
		case 'behind':
			return `${result.org_id}: behind the chain (chain at epoch ${result.chain_epoch})`;
		case 'removed':
			return `${result.org_id}: this device was removed`;
		case 'refused':
			return `${result.org_id}: refused: ${result.reason}`;
		case 'unverified':
			return `${result.org_id}: unverified: ${result.reason}`;
		default: {
			const unknown = result as { org_id: string; result: string };
			return `${unknown.org_id}: unverified: unknown result ${unknown.result}`;
		}
	}
}

/** LLR-hwcj9n: an Organisation's verification status as the list shows it. */
export function verificationLabel(verification: VerificationDto): string {
	switch (verification.status) {
		case 'verified':
			return 'verified';
		case 'unverified':
			return `unverified: ${verification.reason}`;
		default:
			return 'not checked';
	}
}
