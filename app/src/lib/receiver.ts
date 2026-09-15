export type Unlisten = () => void;
export type Subscribe = (cb: (payload: unknown) => void) => Promise<Unlisten>;

/**
 * REQ-rq8g2v. Registers every subscription, then resolves to one cleanup that
 * cancels exactly those registered — including any whose `listen` call resolved
 * after teardown began.
 *
 * PR-u34uqm: the component previously returned its cleanup synchronously while
 * the `listen` calls it was meant to cancel were still pending, so the cleanup
 * emptied an array the pending subscriptions then refilled, leaving listeners
 * attached to a destroyed component and reachable by nothing.
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
