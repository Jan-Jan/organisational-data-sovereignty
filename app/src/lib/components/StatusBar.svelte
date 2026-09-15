<script lang="ts">
	/**
	 * REQ-645jq9: the transport mode the backend is actually running is shown
	 * here, because it changes what the operator must supply elsewhere (the peer
	 * address in Revoke) and was previously invisible.
	 *
	 * REQ-e4ah9h / REQ-bvx4nh: `chain_ws` and `contract_h160` come from the
	 * configuration the backend built at startup, not from the environment, and
	 * are null exactly when `chain_configured` is false. The guards below are
	 * therefore a rendering guard, not a second decision — this component never
	 * infers "configured" from the presence of an endpoint.
	 */
	import { connectionStatus, type ConnectionStatus } from '$lib/api';

	let status = $state<ConnectionStatus | null>(null);
	let error = $state('');

	async function refresh() {
		try {
			status = await connectionStatus();
			error = '';
		} catch (e) {
			error = String(e);
		}
	}

	$effect(() => {
		refresh();
	});
</script>

<div class="status-bar">
	<span class="label">Status:</span>
	{#if error}
		<span class="err">connection_status error: {error}</span>
	{:else if status}
		<span class:ok={status.chain_configured} class:warn={!status.chain_configured}>
			{status.chain_configured ? 'Chain OK' : 'Chain NOT configured'}
		</span>
		{#if status.chain_configured}
			<span class="detail">ws: {status.chain_ws}</span>
			<span class="detail">contract: {status.contract_h160}</span>
		{/if}
		<!-- REQ-645jq9 -->
		<span class="detail">transport: {status.transport_mode}</span>
		<span class="detail">data: {status.data_dir}</span>
	{:else}
		<span class="muted">loading…</span>
	{/if}
	<button onclick={refresh} style="margin-left:0.5rem;font-size:0.75rem;">refresh</button>
</div>

<style>
	.status-bar {
		background: #1a1a2e;
		color: #ccc;
		padding: 0.3rem 0.75rem;
		font-size: 0.8rem;
		display: flex;
		gap: 0.75rem;
		align-items: center;
		border-bottom: 1px solid #333;
		flex-wrap: wrap;
	}
	.label { font-weight: bold; color: #aaa; }
	.ok { color: #4caf50; font-weight: bold; }
	.warn { color: #ff9800; font-weight: bold; }
	.err { color: #f44336; }
	.detail { color: #888; font-family: monospace; font-size: 0.75rem; }
	.muted { color: #555; }
	button { cursor: pointer; background: #333; color: #ccc; border: 1px solid #555; border-radius: 3px; padding: 0.1rem 0.4rem; }
	button:hover { background: #444; }
</style>
