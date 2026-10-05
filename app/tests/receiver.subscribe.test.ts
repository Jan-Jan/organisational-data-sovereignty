/**
 * Registering and tearing down the verification view's event listeners.
 *
 * REQ-rq8g2v — unsubscribe exactly the listeners registered, including those
 *              whose registration completed after teardown began.
 * PR-u34uqm  — the defect that clause was written for.
 */

import { describe, it, expect } from 'vitest';
import { subscribeAll } from '../src/lib/receiver';
import type { Subscribe } from '../src/lib/receiver';

const noop = () => {};

describe('subscribeAll', () => {
	// verifies: LLR-z4ky6f
	it('registers every subscription it is given', async () => {
		const registered: string[] = [];
		const make = (name: string): Subscribe => async () => {
			registered.push(name);
			return () => {};
		};
		await subscribeAll([
			[make('a'), noop],
			[make('b'), noop],
			[make('c'), noop]
		]);
		expect(registered.sort()).toEqual(['a', 'b', 'c']);
	});

	// verifies: LLR-z4ky6f, PR-u34uqm
	it('unsubscribes exactly the listeners it registered, including those whose listen call resolved late', async () => {
		const cancelled: string[] = [];
		const fast: Subscribe = async () => () => cancelled.push('fast');
		const slow: Subscribe = async () => {
			await new Promise((r) => setTimeout(r, 20));
			return () => cancelled.push('slow');
		};
		const cleanup = await subscribeAll([
			[fast, noop],
			[slow, noop]
		]);
		cleanup();
		// Against the old shape the slow listener was registered into an array the
		// cleanup had already emptied, so it was never cancelled.
		expect(cancelled.sort()).toEqual(['fast', 'slow']);
	});

	// verifies: LLR-z4ky6f
	it('resolves only after every subscription has registered', async () => {
		let registered = false;
		const slow: Subscribe = async () => {
			await new Promise((r) => setTimeout(r, 20));
			registered = true;
			return () => {};
		};
		const pending = subscribeAll([[slow, noop]]);
		expect(registered).toBe(false);
		await pending;
		expect(registered).toBe(true);
	});

	// verifies: LLR-z4ky6f
	it('is idempotent — calling the cleanup twice unsubscribes once', async () => {
		let cancels = 0;
		const one: Subscribe = async () => () => {
			cancels += 1;
		};
		const cleanup = await subscribeAll([[one, noop]]);
		cleanup();
		cleanup();
		cleanup();
		expect(cancels).toBe(1);
	});

	// verifies: LLR-z4ky6f
	it('propagates a subscription failure rather than resolving a partial cleanup', async () => {
		const ok: Subscribe = async () => () => {};
		const broken: Subscribe = async () => {
			throw new Error('listen refused');
		};
		// A partial cleanup would silently leave the caller believing every
		// listener is attached when only some are.
		await expect(
			subscribeAll([
				[ok, noop],
				[broken, noop]
			])
		).rejects.toThrow('listen refused');
	});

	// verifies: PR-cu2h2g
	// Pins TODAY's defective behaviour, which the fix of PR-cu2h2g must change:
	// when one registration rejects, a listener that had already registered is
	// never cancelled, and no cleanup is returned that could cancel it.
	it('leaves a registered listener attached when another registration fails (PR-cu2h2g)', async () => {
		const cancelled: string[] = [];
		const ok: Subscribe = async () => () => cancelled.push('ok');
		const broken: Subscribe = async () => {
			await new Promise((r) => setTimeout(r, 5));
			throw new Error('listen refused');
		};
		await expect(
			subscribeAll([
				[ok, noop],
				[broken, noop]
			])
		).rejects.toThrow('listen refused');
		await new Promise((r) => setTimeout(r, 20));
		expect(cancelled).toEqual([]);
	});

	// verifies: LLR-z4ky6f
	it('registers nothing and cleans up cleanly when given no subscriptions', async () => {
		const cleanup = await subscribeAll([]);
		expect(() => cleanup()).not.toThrow();
		expect(() => cleanup()).not.toThrow();
	});
});
