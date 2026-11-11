/**
 * The contract of the faked Tauri boundary: `./contract/backend.json`, the
 * one fixture this file and apps/desktop/src-tauri/tests/ipc.rs both read. The fake
 * (./fake-tauri.ts) refuses to answer, refuse or emit a value whose shape no
 * entry of the fixture has; ipc.rs drives the real command or event for
 * every entry and requires the same shape. So the fake cannot drift from the
 * backend without one side failing.
 */
import backend from './contract/backend.json';

type Entries = Record<string, Record<string, unknown>>;

/** The keys whose value is a tag the webview branches on: compared by value. */
const TAGS = new Set(['result', 'status', 'kind', 'transport_mode', 'write']);
const HEX_ID = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/;

/**
 * Where `actual` departs from the shape of `expected` (the rule
 * backend.json's `about` states), or `null` when it has the same shape.
 * The same rule as `same_shape` in apps/desktop/src-tauri/tests/ipc.rs.
 */
export function shapeMismatch(expected: unknown, actual: unknown, path = '$', key = ''): string | null {
	if (expected === null || actual === null) return expected === actual ? null : `${path}: ${String(actual)} for ${String(expected)}`;
	if (Array.isArray(expected) || Array.isArray(actual)) {
		if (!Array.isArray(expected) || !Array.isArray(actual)) return `${path}: an array on one side only`;
		if (expected.length === 0 && actual.length > 0) return `${path}: elements where none are expected`;
		for (const [index, item] of actual.entries()) {
			if (!expected.some((other) => shapeMismatch(other, item, `${path}[${index}]`, key) === null)) {
				return `${path}[${index}]: no element of that shape is expected`;
			}
		}
		return null;
	}
	if (typeof expected !== typeof actual) return `${path}: ${typeof actual} for ${typeof expected}`;
	if (typeof expected === 'object') {
		const want = Object.keys(expected as object).sort();
		const got = Object.keys(actual as object).sort();
		if (want.join() !== got.join()) return `${path}: keys ${got.join(',')} for ${want.join(',')}`;
		for (const name of want) {
			const inner = shapeMismatch(
				(expected as Record<string, unknown>)[name],
				(actual as Record<string, unknown>)[name],
				`${path}.${name}`,
				name
			);
			if (inner !== null) return inner;
		}
		return null;
	}
	if (typeof expected === 'boolean') return expected === actual ? null : `${path}: ${String(actual)} for ${String(expected)}`;
	if (typeof expected === 'string') {
		if (TAGS.has(key) && expected !== actual) return `${path}: tag ${String(actual)} for ${expected}`;
		if (HEX_ID.test(expected) && !(HEX_ID.test(actual as string) && (actual as string).length === expected.length)) {
			return `${path}: ${String(actual)} is not ${expected.length} hexadecimal characters`;
		}
	}
	return null;
}

function requireEntry(entries: Entries, group: string, name: string, value: unknown) {
	const variants = entries[name] ?? entries['*'];
	if (variants === undefined) throw new Error(`backend.json has no ${group} entry for ${name}`);
	const mismatches = Object.entries(variants).map(([variant, expected]) => `${variant}: ${shapeMismatch(expected, value)}`);
	if (Object.values(variants).some((expected) => shapeMismatch(expected, value) === null)) return;
	throw new Error(`the fake's ${group} for ${name} has no shape backend.json holds (${mismatches.join('; ')}): ${JSON.stringify(value)}`);
}

/** Throws unless `value` has the shape of one answer backend.json holds for `command`. */
export function requireAnswer(command: string, value: unknown) {
	requireEntry(backend.answers as unknown as Entries, 'answer', command, value);
}

/** Throws unless `value` has the shape of one refusal backend.json holds for `command` (or of any other command's). */
export function requireRefusal(command: string, value: unknown) {
	requireEntry(backend.refusals as unknown as Entries, 'refusal', command, value);
}

/** Throws unless `payload` has the shape of one payload backend.json holds for `event`. */
export function requireEvent(event: string, payload: unknown) {
	requireEntry(backend.events as unknown as Entries, 'event', event, payload);
}
