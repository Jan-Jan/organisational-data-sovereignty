<script lang="ts">
	/**
	 * Story 4: Live membership status.
	 * Subscribes to membership-updated / incoming-verified / epoch-changed /
	 * receiver-error / revoked / verification-failed / record-unreadable /
	 * receiver-stopped events, and calls start_receiver to kick off the loop.
	 * THE KEY PoC RESULT: epoch + verified-root match ✓/✗ is displayed prominently.
	 *
	 * REQ-a83vqr, REQ-akt4p7: the ✓/✗ verdict is NOT decided here. Every row is
	 * built by `verifyResultFrom`, which is gated by tests. This component
	 * previously wrote `verified: true` at both of its assignment sites, so the ✗
	 * state was unreachable by any input (HAZ-9fmhm4).
	 */
	import {
		startReceiver,
		listOrgs,
		onMembershipUpdated,
		onIncomingVerified,
		onEpochChanged,
		onReceiverError,
		onRevoked,
		onVerificationFailed,
		onRecordUnreadable,
		onReceiverStopped,
		type MembershipUpdatedPayload,
		type VerificationFailedPayload,
		type RecordUnreadablePayload,
		type ReceiverStoppedPayload,
		type RevokedPayload,
		type ReceiverErrorPayload,
		type OrgDto
	} from '$lib/api';
	import {
		applyReceiverEvent,
		type ReceiverAnnouncement,
		type VerifyRow
	} from '$lib/verify';
	import { subscribeAll, type Unlisten } from '$lib/receiver';

	let receiverStarted = $state(false);
	let receiverErr = $state('');
	let startBusy = $state(false);
	/** REQ-jfxah3: why the loop stopped, once it has. */
	let receiverStoppedReason = $state('');

	let verifyLog = $state<VerifyRow[]>([]);
	let receiverErrors = $state<string[]>([]);
	/** REQ-2k7ys4 / REQ-dp95pv: verified updates whose record could not be read back. */
	let unreadableRecords = $state<string[]>([]);
	let orgs = $state<OrgDto[]>([]);
	let revokedOrgs = $state<string[]>([]);

	interface Props {
		notifyReload?: () => void;
	}
	let { notifyReload }: Props = $props();

	async function loadOrgs() {
		try { orgs = await listOrgs(); } catch { /* ignore */ }
	}

	function now(): string {
		return new Date().toLocaleTimeString();
	}

	/**
	 * REQ-wu6z9p / RC-3rddh7: which list an announcement lands in is decided by
	 * `applyReceiverEvent`, which is gated by tests — not here. This component
	 * only assigns the result back into its state.
	 */
	function announce(event: ReceiverAnnouncement) {
		const next = applyReceiverEvent({ verifyLog, receiverErrors }, event, now());
		verifyLog = next.verifyLog;
		receiverErrors = next.receiverErrors;
	}

	function recordVerified(p: MembershipUpdatedPayload) {
		announce({ kind: 'verified', org_id: p.org_id, epoch: p.epoch, root: p.root });
		loadOrgs();
	}

	$effect(() => {
		loadOrgs();

		// REQ-rq8g2v / PR-u34uqm: the cleanup cancels exactly the listeners that
		// were registered, including any whose `listen` call resolved after
		// teardown began. The previous shape pushed into an array the synchronous
		// cleanup had already emptied, leaving live listeners bound to a destroyed
		// component and reachable by nothing.
		let torndown = false;
		let cleanup: Unlisten | null = null;

		subscribeAll([
			[onMembershipUpdated, recordVerified],
			[onIncomingVerified, recordVerified],
			[
				onEpochChanged,
				() => {
					// Update org list when epoch changes.
					loadOrgs();
				}
			],
			[
				// REQ-wu6z9p, REQ-kn5rtx: a chain read or transport failure. It is
				// not a verdict on any update, so it is rendered here — outside the
				// verification log — and produces no ✗ row (HAZ-9fmhm4).
				onReceiverError,
				(p: ReceiverErrorPayload) => {
					announce({ kind: 'receiver-error', message: p.message });
				}
			],
			[
				onRevoked,
				(p: RevokedPayload) => {
					revokedOrgs = [p.org_id, ...revokedOrgs];
					loadOrgs();
				}
			],
			[
				// REQ-affyf5, REQ-a83vqr: a failure produces a row that says so. This
				// is the only producer of a ✗ row, and before this change there was
				// none (HAZ-9fmhm4).
				onVerificationFailed,
				(p: VerificationFailedPayload) => {
					announce({ kind: 'failed', org_id: p.org_id, message: p.message });
				}
			],
			[
				// REQ-2k7ys4, REQ-dp95pv: the update verified but the record could not
				// be read back, so there is no epoch and no root to show — and in
				// particular not a literal 0, which is genesis (HAZ-5ha5vv).
				onRecordUnreadable,
				(p: RecordUnreadablePayload) => {
					unreadableRecords = [
						`[${now()}] org …${p.org_id.slice(-12)} — update verified, but its record could not be read back (no epoch or root known)`,
						...unreadableRecords.slice(0, 19)
					];
				}
			],
			[
				// REQ-jfxah3: the loop has exited. The badge is cleared, because it
				// claimed a loop was running and one is not (HAZ-cfp4jb).
				onReceiverStopped,
				(p: ReceiverStoppedPayload) => {
					receiverStarted = false;
					receiverStoppedReason = p.reason;
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

	async function doStartReceiver() {
		startBusy = true;
		receiverErr = '';
		receiverStoppedReason = '';
		try {
			await startReceiver();
			receiverStarted = true;
		} catch (e) {
			receiverErr = String(e);
		} finally {
			startBusy = false;
		}
	}

	function orgLabel(orgId: string): string {
		const o = orgs.find((x) => x.org_id === orgId);
		if (o) return `…${orgId.slice(-12)} (epoch ${o.epoch})`;
		return `…${orgId.slice(-12)}`;
	}
</script>

<section class="panel">
	<h2>Story 4: Live Membership &amp; Chain Verification</h2>
	<p class="hint">
		Start the receiver loop to watch for inbound org updates.
		Each update is verified against the on-chain root — the result (✓/✗) is the core PoC output.
	</p>

	<div class="receiver-ctl">
		{#if !receiverStarted}
			<button onclick={doStartReceiver} disabled={startBusy}>
				{startBusy ? 'Starting…' : 'Start Receiver Loop'}
			</button>
			{#if receiverErr}<p class="err">{receiverErr}</p>{/if}
			{#if receiverStoppedReason}
				<!-- REQ-jfxah3 -->
				<p class="err stopped-badge">Receiver stopped — {receiverStoppedReason}</p>
			{/if}
		{:else}
			<p class="ok receiver-badge">Receiver running — listening for updates…</p>
		{/if}
	</div>

	<!-- THE KEY PoC OUTPUT: verified root log -->
	<div class="verify-section">
		<h3>Verified Updates (chain root match)</h3>
		{#if verifyLog.length === 0}
			<p class="muted">No updates received yet.</p>
		{:else}
			<table>
				<thead>
					<tr><th>Time</th><th>Org</th><th>Epoch</th><th>Root (tail)</th><th>Verified</th><th>Detail</th></tr>
				</thead>
				<tbody>
					{#each verifyLog as v, i (i)}
						<tr>
							<td class="mono small">{v.ts}</td>
							<td class="mono small">{orgLabel(v.org_id)}</td>
							<!-- epoch/root are null on a failure: a failure knows neither,
							     and 0 / "" are real values (HAZ-5ha5vv). -->
							<td>{v.epoch === null ? '—' : v.epoch}</td>
							<td class="mono small">{v.root === null ? '—' : `…${v.root.slice(-16)}`}</td>
							<td class:check={v.verified} class:cross={!v.verified}>
								{v.verified ? '✓ MATCH' : '✗ MISMATCH'}
							</td>
							<td class="small">{v.detail ?? ''}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>

	<!-- Revoked orgs -->
	{#if revokedOrgs.length > 0}
		<div class="revoked-section">
			<h3>Revoked &amp; Self-Deleted</h3>
			{#each revokedOrgs as orgId, i (i)}
				<p class="revoked-entry">Removed from org …{orgId.slice(-12)}</p>
			{/each}
		</div>
	{/if}

	<!-- REQ-dp95pv: verified updates with no readable record -->
	{#if unreadableRecords.length > 0}
		<div class="errors-section">
			<h3>Verified, but Record Unreadable</h3>
			{#each unreadableRecords as msg, i (i)}
				<p class="err small">{msg}</p>
			{/each}
		</div>
	{/if}

	<!-- Receiver errors (non-fatal) -->
	{#if receiverErrors.length > 0}
		<div class="errors-section">
			<h3>Receiver Errors (non-fatal)</h3>
			{#each receiverErrors as msg, i (i)}
				<p class="err small">{msg}</p>
			{/each}
		</div>
	{/if}
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	h3 { font-size: 0.9rem; color: #aaa; margin: 0.75rem 0 0.4rem; }
	.hint { color: #666; font-size: 0.8rem; margin-bottom: 0.75rem; }
	.receiver-ctl { margin-bottom: 0.75rem; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.35rem 0.9rem; font-size: 0.85rem; }
	button:disabled { opacity: 0.5; cursor: default; }
	button:not(:disabled):hover { background: #204d7a; }
	.receiver-badge { background: #0d2f1d; border: 1px solid #2e7d32; padding: 0.35rem 0.75rem; border-radius: 3px; display: inline-block; }
	.stopped-badge { background: #1a0a0a; border: 1px solid #5c1111; padding: 0.35rem 0.75rem; border-radius: 3px; display: inline-block; }

	.verify-section { margin-bottom: 0.75rem; }
	table { border-collapse: collapse; width: 100%; font-size: 0.82rem; }
	th, td { text-align: left; padding: 0.25rem 0.5rem; border-bottom: 1px solid #1a1a1a; }
	th { color: #666; font-weight: normal; }
	td { color: #bbb; }
	.mono { font-family: monospace; }
	.small { font-size: 0.75rem; color: #777; }

	/* THE KEY DISPLAY: verified/not */
	.check { color: #4caf50; font-weight: bold; font-size: 0.9rem; }
	.cross { color: #f44336; font-weight: bold; font-size: 0.9rem; }

	.revoked-section { background: #1a0a0a; border: 1px solid #5c1111; border-radius: 4px; padding: 0.5rem 0.75rem; margin-bottom: 0.5rem; }
	.revoked-entry { color: #f44336; font-size: 0.82rem; margin: 0.2rem 0; }

	.errors-section { background: #1a1200; border: 1px solid #4a3800; border-radius: 4px; padding: 0.5rem 0.75rem; margin-bottom: 0.5rem; }
	.err { color: #f44336; }
	.ok { color: #4caf50; }
	.muted { color: #555; font-size: 0.82rem; }
</style>
