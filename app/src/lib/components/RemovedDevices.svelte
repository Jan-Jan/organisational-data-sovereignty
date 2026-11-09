<script lang="ts">
	/**
	 * SDD-g9ugu7: the removed Devices view. org-io lists each Device the
	 * Organisation's store holds as revoked; this panel words each one
	 * (LLR-cs9bmw) and reads the list when shown, on Refresh, and after a
	 * verification result for its Organisation (LLR-x9qbhu).
	 */
	import { listRemovedDevices, onVerificationResult, type RemovedDeviceDto, type VerificationEntryDto } from '$lib/api';
	import { removedDeviceLine } from '$lib/removed';
	import { subscribeAll, type Unlisten } from '$lib/receiver';

	interface Props {
		orgId: string;
	}
	let { orgId }: Props = $props();

	let removedDevices = $state<RemovedDeviceDto[]>([]);
	let refusalText = $state('');
	let listeningRefusal = $state('');
	let loaded = $state(false);

	// Reads can overlap and their answers land in any order; only the
	// latest read's answer is shown (LLR-x9qbhu, 2026-11-09 note).
	let latestRead = 0;

	async function load() {
		const read = ++latestRead;
		try {
			const answer = await listRemovedDevices(orgId);
			if (read !== latestRead) return;
			removedDevices = answer;
			refusalText = '';
		} catch (refusal) {
			if (read !== latestRead) return;
			refusalText = String(refusal);
		}
		loaded = true;
	}

	$effect(() => {
		load();

		// The same teardown as Revoke's (REQ-rq8g2v): a subscription still
		// pending at teardown is cancelled once it resolves.
		let torndown = false;
		let cleanup: Unlisten | null = null;
		subscribeAll([
			[
				onVerificationResult,
				(entry: VerificationEntryDto) => {
					if (entry.org_id === orgId) load();
				}
			]
		]).then(
			(unlisten) => {
				if (torndown) unlisten();
				else cleanup = unlisten;
			},
			(refusal) => {
				if (!torndown) listeningRefusal = `listening for verification results failed: ${String(refusal)}`;
			}
		);

		return () => {
			torndown = true;
			cleanup?.();
		};
	});
</script>

<section class="panel">
	<h2>Removed Devices (Admin)</h2>
	<p class="hint">
		An acknowledgement is the removed Device's own signed statement that it acted on its removal. It is not proof that its data was deleted.
	</p>
	<button onclick={load}>Refresh</button>
	{#if listeningRefusal}
		<p class="err">{listeningRefusal}</p>
	{/if}

	{#if refusalText}
		<p class="err">{refusalText}</p>
	{:else if loaded && removedDevices.length === 0}
		<p class="muted">No removed Devices.</p>
	{:else}
		<ul>
			{#each removedDevices as removed (removed.device_key)}
				<li>
					<span class="mono">Member {removed.member_id}</span>
					<span class="mono">Device {removed.device_key}</span>
					<span>{removedDeviceLine(removed)}</span>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	.hint { color: #666; font-size: 0.8rem; margin-bottom: 0.75rem; }
	ul { list-style: none; padding: 0; margin: 0.5rem 0 0; }
	li { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.82rem; margin-bottom: 0.5rem; }
	.mono { font-family: monospace; color: #aaa; word-break: break-all; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.35rem 0.9rem; font-size: 0.85rem; }
	.err { color: #f44336; font-size: 0.8rem; }
	.muted { color: #555; font-size: 0.82rem; }
</style>
