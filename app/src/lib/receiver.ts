export type Unlisten = () => void;
export type Subscribe = (cb: (payload: unknown) => void) => Promise<Unlisten>;

/**
 * REQ-rq8g2v. Registers every subscription, then resolves to one cleanup that
 * cancels exactly those registered — including any whose `listen` call resolved
 * after teardown began.
 *
 * The defect this replaced: a cleanup returned synchronously while the `listen` calls it is
 * meant to cancel are still pending empties an array the pending
 * subscriptions then refill, leaving listeners attached to a destroyed
 * component and reachable by nothing.
 */
export async function subscribeAll(
	subscriptions: Array<[Subscribe, (payload: never) => void]>
): Promise<Unlisten> {
	const unlisteners = await Promise.all(
		subscriptions.map(([subscribe, handler]) => subscribe(handler as (p: unknown) => void))
	);
	let done = false;
	return () => {
		if (done) return;
		done = true;
		for (const u of unlisteners) u();
	};
}
