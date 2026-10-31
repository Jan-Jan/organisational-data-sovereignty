<script lang="ts">
	/**
	 * Story 1: propose the Person and found an Organisation.
	 * Form → propose_person (LLR-k57uqv; passed over when a Person is held
	 * already) → propose_founding_organisation (LLR-6ff735). Shows the new
	 * Persona and its founder's Device key, and hands the founder to the
	 * founding-writes list, where "Write to chain" writes it (LLR-rhc555).
	 * Nothing is written to the chain here. Whether a refused Person stops
	 * the founding is decided by `personRefusalStep` (SDD-g5wmxt,
	 * LLR-d8u93q); this component makes the IPC calls.
	 */
	import { proposePerson, proposeFoundingOrganisation, type PersonDto } from '$lib/api';
	import { personRefusalStep, type FoundingProposals } from '$lib/founding';

	interface Props {
		notifyReload?: () => void;
		/** LLR-rhc555: receives the founder of each founding proposed here. */
		onProposed?: (founder: string) => void;
	}
	let { notifyReload, onProposed }: Props = $props();

	let handle = $state('');
	let name = $state('');
	let surname = $state('');
	let orgName = $state('');
	let busy = $state(false);
	let proposed = $state<FoundingProposals | null>(null);
	let err = $state('');

	async function run() {
		busy = true;
		err = '';
		try {
			// LLR-k57uqv: the Person first; a Person held already is passed over.
			let person: PersonDto | null = null;
			try {
				person = await proposePerson(name.trim(), surname.trim());
			} catch (refusal) {
				const step = personRefusalStep(refusal);
				if (!step.proceed) {
					err = step.message;
					return;
				}
			}
			// LLR-6ff735: then the founding.
			const founding = await proposeFoundingOrganisation(name.trim(), surname.trim(), handle.trim(), orgName.trim());
			proposed = { person, founding };
			onProposed?.(founding.founder);
			notifyReload?.();
		} catch (refusal) {
			// org-io's refusal of the founding, naming the field (LLR-6ff735).
			err = String(refusal);
		} finally {
			busy = false;
		}
	}

	function reset() {
		handle = '';
		name = '';
		surname = '';
		orgName = '';
		proposed = null;
		err = '';
	}
</script>

<section class="panel">
	<h2>Story 1: Propose Person + founding</h2>
	<p class="hint">Proposes your Person (if none is held yet) and a founding; write it to the chain from Personas / Orgs.</p>

	{#if proposed}
		<div class="result-box">
			<p>Founding proposed: provisional until it is written to the chain.</p>
			{#if proposed.person}<p>Person proposed: <code>{proposed.person.device_public_key}</code></p>{/if}
			<p>Persona ID: <code>{proposed.founding.persona_id}</code></p>
			<p>Founder's DevicePublicKey: <code>{proposed.founding.founder}</code></p>
		</div>
		<button onclick={reset}>Propose another</button>
	{:else}
		<form onsubmit={(e) => { e.preventDefault(); run(); }}>
			<label>
				Handle
				<input type="text" bind:value={handle} placeholder="alice" disabled={busy} />
			</label>
			<label>
				Name
				<input type="text" bind:value={name} placeholder="Alice" disabled={busy} />
			</label>
			<label>
				Surname
				<input type="text" bind:value={surname} placeholder="Smith" disabled={busy} />
			</label>
			<label>
				Organisation name
				<input type="text" bind:value={orgName} placeholder="Acme" disabled={busy} />
			</label>

			{#if err}
				<p class="err">{err}</p>
			{/if}

			<button type="submit" disabled={busy}>
				{busy ? 'Proposing…' : 'Propose Person + founding'}
			</button>
		</form>
	{/if}
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	.hint { color: #666; font-size: 0.8rem; margin-bottom: 0.75rem; }
	form { display: flex; flex-direction: column; gap: 0.5rem; }
	label { display: flex; flex-direction: column; gap: 0.2rem; font-size: 0.85rem; color: #aaa; }
	input { background: #1a1a1a; border: 1px solid #333; color: #eee; padding: 0.3rem 0.5rem; border-radius: 3px; font-size: 0.9rem; }
	input:disabled { opacity: 0.5; }
	button { margin-top: 0.25rem; cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.4rem 1rem; font-size: 0.85rem; align-self: flex-start; }
	button:disabled { opacity: 0.5; cursor: default; }
	button:not(:disabled):hover { background: #204d7a; }
	.result-box { background: #0d2f1d; border: 1px solid #1b5e20; border-radius: 4px; padding: 0.75rem; margin-bottom: 0.5rem; }
	.result-box p { margin: 0.2rem 0; font-size: 0.85rem; color: #ccc; }
	code { font-family: monospace; font-size: 0.8rem; color: #81c784; word-break: break-all; }
	.err { color: #f44336; font-size: 0.85rem; }
</style>
