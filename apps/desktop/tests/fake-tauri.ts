/**
 * The Tauri boundary, faked for rendered tests: `invoke` answers each
 * command as the backend would (or refuses it with its text, as Tauri
 * rejects a command's `Err`), and `listen` keeps each handler so a test can
 * emit the event the backend would. Everything above it, `src/lib/api.ts`
 * and every component, runs for real.
 *
 * A test file routes the two modules here:
 *
 *     vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
 *     vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));
 */
import { vi } from 'vitest';
import { requireAnswer, requireEvent, requireRefusal } from './contract';

type Args = Record<string, unknown> | undefined;
type Answer = (args: Args) => unknown;
type Handler = (event: { event: string; payload: unknown }) => void;

/** What the page's start asks for, answered as an empty installation with no chain. */
const START: Record<string, Answer> = {
	connection_status: () => ({
		chain_configured: false,
		chain_ws: null,
		contract_h160: null,
		transport_mode: 'loopback',
		data_dir: '/tmp/ods'
	}),
	list_personas: () => [],
	list_orgs: () => [],
	verify_organisation: () => [],
	list_admission_requests: () => []
};

const answers = new Map<string, Answer>();
const listeners = new Map<string, Handler[]>();
let refusedEvents: string | null = null;

const invoke = vi.fn(async (command: string, args?: Record<string, unknown>): Promise<unknown> => {
	const answer = answers.get(command);
	if (answer === undefined) throw `the fake backend has no answer for ${command}`;
	const value = await answer(args);
	requireAnswer(command, value);
	return value;
});

const listen = vi.fn(async (event: string, handler: Handler): Promise<() => void> => {
	if (refusedEvents !== null) throw refusedEvents;
	listeners.set(event, [...(listeners.get(event) ?? []), handler]);
	return () => listeners.set(event, (listeners.get(event) ?? []).filter((held) => held !== handler));
});

export const tauri = {
	invoke,
	listen,
	/** Forget every answer, listener and call, then answer the page's start. */
	reset() {
		invoke.mockClear();
		listen.mockClear();
		answers.clear();
		listeners.clear();
		refusedEvents = null;
		for (const [command, answer] of Object.entries(START)) answers.set(command, answer);
	},
	/** `command` returns `value`, or what `value` returns from the call's arguments. */
	answer(command: string, value: unknown) {
		answers.set(command, typeof value === 'function' ? (value as Answer) : () => value);
	},
	/**
	 * `command` stays in flight until the test settles it: with an answer
	 * (checked as `answer`'s are) or a refusal (checked as `refuse`'s are).
	 */
	defer(command: string): { answer(value: unknown): void; refuse(refusal: unknown): void } {
		let resolve!: (value: unknown) => void;
		let reject!: (refusal: unknown) => void;
		const inFlight = new Promise<unknown>((onAnswer, onRefusal) => {
			resolve = onAnswer;
			reject = onRefusal;
		});
		answers.set(command, () => inFlight);
		return {
			answer: resolve,
			refuse(refusal: unknown) {
				requireRefusal(command, refusal);
				reject(refusal);
			}
		};
	},
	/** `command` is refused with `refusal`, as Tauri rejects a command's `Err`: its text, or `propose_person`'s kind and text. */
	refuse(command: string, refusal: unknown) {
		requireRefusal(command, refusal);
		answers.set(command, () => {
			throw refusal;
		});
	},
	/** Every `listen` call is refused with `text`. */
	refuseListening(text: string) {
		refusedEvents = text;
	},
	/** The backend emits `event` with `payload` to every listener registered for it. */
	emit(event: string, payload: unknown) {
		requireEvent(event, payload);
		for (const handler of listeners.get(event) ?? []) handler({ event, payload });
	},
	/** The arguments of each call of `command`, in order (`undefined` where it was invoked with none). */
	calls(command: string): Args[] {
		return invoke.mock.calls.filter(([name]) => name === command).map(([, args]) => args);
	},
	/** The commands invoked, in order. */
	commands(): string[] {
		return invoke.mock.calls.map(([name]) => name);
	}
};
