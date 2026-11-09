<script lang="ts">
	/**
	 * Story 5: Revoke a member.
	 * A: show member list → enter member_id_hex → "Propose revocation" → propose_revocation.
	 * B: shows that this Device is removed when the "revoked" event fires, which a
	 *    Verification's removed result raises (LLR-ygr6gd; monitored via Membership).
	 *
	 * REQ-he8ejb: the precondition is `validateRevokeInput`'s, gated by tests; it
	 * takes the member identifier alone. The revocation is recorded as a
	 * provisional change and goes with the next epoch written (LLR-7n4hgr),
	 * which is what the panel says once it is proposed (LLR-b59vcx).
	 */
	import { listOrgs, proposeRevocation, onRevoked, type OrgDto, type RevokedPayload } from '$lib/api';
	import { validateRevokeInput } from '$lib/revoke';
	import { NEXT_EPOCH_NOTE } from '$lib/epoch';
	import { subscribeAll, type Unlisten } from '$lib/receiver';
	import RemovedDevices from '$lib/components/RemovedDevices.svelte';

	interface Props {
		notifyReload?: () => void;
	}
	let { notifyReload }: Props = $props();

	let orgs = $state<OrgDto[]>([]);
	let selectedOrgId = $state('');
	let memberIdHex = $state('');

	let revokeBusy = $state(false);
	let revokeErr = $state('');
	let revokeDone = $state(false);

	let revokedEvents = $state<string[]>([]);

	async function loadOrgs() {
		try {
			orgs = await listOrgs();
			if (orgs.length > 0 && !selectedOrgId) selectedOrgId = orgs[0].org_id;
		} catch (e) {
			revokeErr = String(e);
		}
	}

	$effect(() => {
		loadOrgs();

		// REQ-rq8g2v / PR-u34uqm: the cleanup cancels the subscription even when
		// the component is destroyed while `listen` is still pending. Assigning
		// `unlisten` after an un-awaited `await` left the teardown with nothing to
		// call and the listener attached to a destroyed component.
		let torndown = false;
		let cleanup: Unlisten | null = null;

		subscribeAll([
			[
				onRevoked,
				(p: RevokedPayload) => {
					revokedEvents = [
						`${p.org_id === null ? 'A provisional founding' : `Org …${p.org_id.slice(-12)}`} — this Device is removed: a Verification found it absent from the adopted Membership record`,
						...revokedEvents
					];
				}
			]
		]).then((fn) => {
			if (torndown) fn();
			else cleanup = fn;
		});

		return () => {
			torndown = true;
			cleanup?.();
		};
	});

	async function doRevoke() {
		if (!selectedOrgId) {
			revokeErr = 'Select an org.';
			return;
		}
		// REQ-he8ejb. One decision, in one gated place.
		const check = validateRevokeInput({ memberIdHex });
		if (!check.ok) {
			revokeErr = check.message;
			return;
		}
		revokeBusy = true;
		revokeErr = '';
		try {
			await proposeRevocation(selectedOrgId, memberIdHex.trim());
			revokeDone = true;
			notifyReload?.();
		} catch (e) {
			revokeErr = String(e);
		} finally {
			revokeBusy = false;
		}
	}

	function reset() {
		memberIdHex = '';
		revokeErr = '';
		revokeDone = false;
	}
</script>

<section class="panel">
	<h2>Story 5: Revoke Member</h2>
	<p class="hint">
		Enter the member ID (64 hex chars). The revocation is recorded as a provisional change; it
		reaches the chain only when the next epoch is proposed and written on S4: Membership.
	</p>

	{#if revokeDone}
		<div class="result-box ok">
			<p>Revocation proposed. {NEXT_EPOCH_NOTE}</p>
		</div>
		<button onclick={reset}>Revoke another</button>
	{:else}
		<label>
			Org
			{#if orgs.length === 0}
				<span class="muted">No orgs (create one in Story 1)</span>
			{:else}
				<select bind:value={selectedOrgId}>
					{#each orgs as o (o.org_id)}
						<option value={o.org_id}>…{o.org_id.slice(-12)} (epoch {o.epoch}, {o.member_count} members)</option>
					{/each}
				</select>
			{/if}
		</label>

		<label>
			Member ID (64 hex chars)
			<input
				type="text"
				bind:value={memberIdHex}
				placeholder="0000…(64 hex chars)"
				disabled={revokeBusy}
				class="mono"
			/>
		</label>

		{#if revokeErr}<p class="err">{revokeErr}</p>{/if}

		<button class="revoke-btn" onclick={doRevoke} disabled={revokeBusy || orgs.length === 0}>
			{revokeBusy ? 'Proposing…' : 'Propose revocation'}
		</button>
	{/if}

	{#if selectedOrgId}
		{#key selectedOrgId}
			<RemovedDevices orgId={selectedOrgId} />
		{/key}
	{/if}

	<!-- Live revoked events -->
	{#if revokedEvents.length > 0}
		<div class="revoked-log">
			<h3>Revoked events received</h3>
			{#each revokedEvents as msg, i (i)}
				<p class="revoked-entry">{msg}</p>
			{/each}
		</div>
	{/if}
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	h3 { font-size: 0.9rem; color: #aaa; margin: 0.75rem 0 0.4rem; }
	.hint { color: #666; font-size: 0.8rem; margin-bottom: 0.75rem; }
	label { display: flex; flex-direction: column; gap: 0.2rem; font-size: 0.82rem; color: #888; margin-bottom: 0.4rem; }
	input, select { background: #1a1a1a; border: 1px solid #333; color: #eee; padding: 0.3rem 0.5rem; border-radius: 3px; font-size: 0.82rem; }
	input:disabled { opacity: 0.5; }
	.mono { font-family: monospace; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.35rem 0.9rem; font-size: 0.85rem; margin-top: 0.25rem; }
	button:disabled { opacity: 0.5; cursor: default; }
	button:not(:disabled):hover { background: #204d7a; }
	.revoke-btn { background: #4a1010; color: #f44336; border-color: #7a1a1a; }
	.revoke-btn:not(:disabled):hover { background: #5c1414; }
	.result-box { background: #0d2f1d; border: 1px solid #1b5e20; border-radius: 4px; padding: 0.75rem; margin-bottom: 0.5rem; }
	.result-box p { margin: 0.2rem 0; font-size: 0.85rem; color: #ccc; }
	.revoked-log { background: #1a0a0a; border: 1px solid #5c1111; border-radius: 4px; padding: 0.5rem 0.75rem; margin-top: 0.75rem; }
	.revoked-entry { color: #f44336; font-size: 0.82rem; margin: 0.15rem 0; }
	.err { color: #f44336; font-size: 0.8rem; }
	.muted { color: #555; font-size: 0.82rem; }
</style>
