<script lang="ts">
	/**
	 * Story 3: Admit a member (SDD-xg4nj7; markup SDD-6g3wnh's). The kept
	 * admission requests, listed at start and announced on arrival, each shown
	 * beside the invitee name typed for its Invite (LLR-47zvjn). The Admin
	 * chooses one, may choose a Handle (blank: org-io takes the first preferred
	 * Handle free), confirms, and proposes the admission (LLR-e63yrs): it is
	 * recorded as a provisional change, written with the next epoch
	 * (LLR-b59vcx). The Admin may instead cancel the request's Invite
	 * (LLR-e7bbm5, SDD-p7sdy3).
	 */
	import { proposeAdmission } from '$lib/api';
	import { admissionGate, type AdmissionConfirmation } from '$lib/notices';
	import { NEXT_EPOCH_NOTE } from '$lib/epoch';

	interface Props {
		notifyReload?: () => void;
		/** LLR-dhtec7: one confirmation per org id and invite id. */
		requests?: AdmissionConfirmation[];
		/** LLR-e7bbm5: what the page last told of a cancel, and whether one is in flight. */
		cancelMessage?: string;
		cancelBusy?: boolean;
		/** LLR-e7bbm5: the page's cancel, which drops the request of a cancelled Invite; true once cancelled. */
		onCancel?: (confirmation: AdmissionConfirmation) => Promise<boolean>;
	}
	let { notifyReload, requests = [], cancelMessage = '', cancelBusy = false, onCancel }: Props = $props();

	async function doCancel(confirmation: AdmissionConfirmation) {
		const cancelled = (await onCancel?.(confirmation)) ?? false;
		if (cancelled && keyOf(confirmation) === selectedKey) selectedKey = '';
	}

	let selectedKey = $state('');
	let handle = $state('');
	let confirmed = $state(false);
	let admitBusy = $state(false);
	let admitErr = $state('');
	let proposed = $state(false);

	function keyOf(confirmation: AdmissionConfirmation): string {
		return `${confirmation.org_id}/${confirmation.invite_id}`;
	}

	let selected = $derived(requests.find((confirmation) => keyOf(confirmation) === selectedKey) ?? null);
	let gate = $derived(admissionGate({ request: selected, confirmed }));

	function choose(key: string) {
		selectedKey = key;
		confirmed = false;
		admitErr = '';
		proposed = false;
	}

	async function doAdmit() {
		if (!gate.ok) {
			admitErr = gate.message;
			return;
		}
		if (selected === null) return;
		admitBusy = true;
		admitErr = '';
		try {
			await proposeAdmission(selected.org_id, selected.invite_id, handle.trim(), true);
			proposed = true;
			notifyReload?.();
		} catch (e) {
			admitErr = String(e);
		} finally {
			admitBusy = false;
		}
	}
</script>

<section class="panel">
	<h2>Story 3: Admit Member</h2>
	{#if requests.length === 0}
		<p class="muted">No admission requests kept.</p>
	{:else}
		<ul class="requests">
			{#each requests as confirmation (keyOf(confirmation))}
				<li>
					<button class:chosen={keyOf(confirmation) === selectedKey} onclick={() => choose(keyOf(confirmation))}>
						{confirmation.name} {confirmation.surname} — invite …{confirmation.invite_id.slice(-8)}
					</button>
					<button class="cancel-btn" onclick={() => doCancel(confirmation)} disabled={cancelBusy}>Cancel invite</button>
				</li>
			{/each}
		</ul>
	{/if}
	{#if cancelMessage}<p class="cancel-message">{cancelMessage}</p>{/if}

	{#if selected}
		<div class="confirmation">
			<dl>
				<dt>Invited as</dt><dd>{selected.invitee_name}</dd>
				<dt>Request</dt><dd>{selected.name} {selected.surname}</dd>
				<dt>Preferred Handles</dt><dd>{selected.preferred_handles.join(', ')}</dd>
				<dt>DevicePublicKey</dt><dd class="mono">{selected.device_public_key}</dd>
				<dt>Org id</dt><dd class="mono">{selected.org_id}</dd>
			</dl>
			{#if selected.mismatch}
				<p class="warn" role="alert">The name in the request is not the name you typed for this Invite.</p>
			{/if}
			<label>
				Handle (blank: the first preferred Handle free)
				<input type="text" bind:value={handle} disabled={admitBusy} />
			</label>
			<label class="confirm">
				<input type="checkbox" bind:checked={confirmed} disabled={admitBusy} />
				This request is from the person I invited
			</label>
			<button class="admit-btn" onclick={doAdmit} disabled={admitBusy || !gate.ok}>
				{admitBusy ? 'Proposing…' : 'Propose admission'}
			</button>
			{#if !gate.ok}<p class="muted">{gate.message}</p>{/if}
			{#if admitErr}<p class="err">{admitErr}</p>{/if}
			{#if proposed}<p class="ok">Admission proposed. {NEXT_EPOCH_NOTE}</p>{/if}
		</div>
	{/if}
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	.requests { list-style: none; padding: 0; margin: 0 0 0.5rem; }
	label { display: flex; flex-direction: column; gap: 0.2rem; font-size: 0.82rem; color: #888; margin-bottom: 0.4rem; }
	label.confirm { flex-direction: row; align-items: center; gap: 0.4rem; color: #ddd; }
	input { background: #1a1a1a; border: 1px solid #333; color: #eee; padding: 0.3rem 0.5rem; border-radius: 3px; font-family: monospace; font-size: 0.82rem; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.3rem 0.75rem; font-size: 0.82rem; }
	button:disabled { opacity: 0.5; cursor: default; }
	button.chosen { border-color: #7ec8e3; }
	.admit-btn { background: #1b4d1b; color: #81c784; border-color: #2e7d32; margin-top: 0.5rem; }
	.confirmation { background: #0d0d1a; border: 1px solid #222; border-radius: 4px; padding: 0.75rem; margin-top: 0.5rem; }
	dl { display: grid; grid-template-columns: 9rem 1fr; gap: 0.2rem 0.5rem; font-size: 0.82rem; margin: 0 0 0.5rem; }
	dt { color: #666; }
	dd { color: #ccc; margin: 0; word-break: break-all; }
	.mono { font-family: monospace; }
	.err { color: #f44336; font-size: 0.8rem; }
	.ok { color: #4caf50; font-size: 0.85rem; }
	.warn { color: #ff9800; font-size: 0.85rem; }
	.muted { color: #555; font-size: 0.85rem; }
</style>
