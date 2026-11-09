import type { RemovedDeviceDto } from '$lib/api';

/**
 * SDD-g9ugu7, LLR-cs9bmw: the line a removed Device's row shows. An
 * acknowledgement is the Device's own signed statement, so it is worded as
 * that and never as a deletion.
 */
export function removedDeviceLine(removed: RemovedDeviceDto): string {
	if (removed.acknowledged_epoch === null) return 'no acknowledgement received by this Device';
	return `acknowledged its removal at epoch ${removed.acknowledged_epoch}`;
}
