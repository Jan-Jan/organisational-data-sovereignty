<script lang="ts">
	/**
	 * Story 4: Live membership status.
	 * Subscribes to membership-updated / incoming-verified / epoch-changed /
	 * receiver-error / revoked / verification-failed / record-unreadable /
	 * receiver-stopped events, and calls start_receiver to kick off the loop.
	 * THE KEY PoC RESULT: epoch + verified-root match ✓/✗ is displayed prominently.
	 *
	 * REQ-a83vqr, REQ-akt4p7: the ✓/✗ verdict is NOT decided here. Every row is
	 * built by `verifyResultFrom`, which is gated by tests: a row this component
	 * marked `verified: true` itself would make the ✗ state unreachable by any
	 * input (HAZ-9fmhm4).
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
		proposeAdmission,
		proposeRevocation,
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
	import {
		admissionGate,
		reproposalStep,
		type AdmissionConfirmation,
		type AdmissionReproposal,
		type DiscardedNotice,
		type SendReport
	} from '$lib/notices';
	import type { ProposedUpdate } from '$lib/epoch';

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
	let revokedOrgs = $state<(string | null)[]>([]);

	interface Props {
		notifyReload?: () => void;
		/** LLR-md6hhz: the kept admission requests the page lists, for the re-proposal's confirmation. */
		requests?: AdmissionConfirmation[];
		/** LLR-k5rf3r, LLR-cgtav9: the notices for discarded changes the page holds. */
		notices?: DiscardedNotice[];
		/** LLR-9r6y7p, LLR-cgtav9: the send and request reports the page holds. */
		sendReports?: SendReport[];
		onDismissNotice?: (id: string) => void;
		onDismissReport?: (id: string) => void;
		/** LLR-tbtrb3: the proposed updates the page holds. */
		proposedUpdates?: ProposedUpdate[];
		/** LLR-5vgpkd, LLR-tbtrb3: what the page last told of a proposal or a write, and the Organisation in flight. */
		epochMessage?: string;
		epochBusy?: string | null;
		/** LLR-5vgpkd, LLR-tbtrb3: the page's proposal and write, whose outcomes outlive this panel. */
		onProposeNextEpoch?: (orgId: string) => Promise<void>;
		onWriteUpdate?: (orgId: string) => Promise<void>;
	}
	let {
		notifyReload,
		requests = [],
		notices = [],
		sendReports = [],
		onDismissNotice,
		onDismissReport,
		proposedUpdates = [],
		epochMessage = '',
		epochBusy = null,
		onProposeNextEpoch,
		onWriteUpdate
	}: Props = $props();

	async function loadOrgs() {
		try { orgs = await listOrgs(); } catch { /* ignore */ }
	}

	let reproposeErr = $state('');
	/** LLR-md6hhz, LLR-e63yrs: the admission being proposed again, awaiting the Admin's confirmation. */
	let confirming = $state<{ confirmation: AdmissionConfirmation; again: AdmissionReproposal } | null>(
		null
	);
	let confirmedAgain = $state(false);
	let gateAgain = $derived(admissionGate({ request: confirming?.confirmation ?? null, confirmed: confirmedAgain }));

	/** LLR-tbtrb3: "Write to chain" through the page; the Organisations are listed again once it is done. */
	async function writeUpdate(orgId: string) {
		await onWriteUpdate?.(orgId);
		loadOrgs();
	}

	/**
	 * LLR-md6hhz: "Propose again". An admission opens the REQ-88npkt
	 * confirmation (LLR-e63yrs) and proposes nothing yet; a revocation is
	 * proposed again at once. The notice stays until dismissed.
	 */
	async function proposeAgain(notice: DiscardedNotice) {
		reproposeErr = '';
		const step = reproposalStep(notice, requests);
		if (step.step === 'unavailable') {
			reproposeErr = step.message;
			return;
		}
		if (step.step === 'confirm') {
			confirming = { confirmation: step.confirmation, again: step.reproposal };
			confirmedAgain = false;
			return;
		}
		try {
			await proposeRevocation(step.reproposal.org_id, step.reproposal.member_id);
			notifyReload?.();
		} catch (refusal) {
			reproposeErr = String(refusal);
		}
	}

	/** LLR-e63yrs: the admission is proposed again, with confirmed true, only once the Admin confirmed. */
	async function confirmProposeAgain() {
		if (confirming === null) return;
		if (!gateAgain.ok) {
			reproposeErr = gateAgain.message;
			return;
		}
		const again = confirming.again;
		try {
			await proposeAdmission(again.org_id, again.invite_id, again.handle, true);
			confirming = null;
			confirmedAgain = false;
			reproposeErr = '';
			notifyReload?.();
		} catch (refusal) {
			reproposeErr = String(refusal);
		}
	}

	/** Closing the confirmation proposes nothing; the notice stays. */
	function cancelProposeAgain() {
		confirming = null;
		confirmedAgain = false;
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
				// is the only producer of a ✗ row (HAZ-9fmhm4).
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
						`[${now()}] ${p.org_id === null ? 'a provisional founding' : `org …${p.org_id.slice(-12)}`} — update verified, but its record could not be read back (no epoch or root known)`,
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

	<!-- LLR-5vgpkd, LLR-tbtrb3: the next epoch proposed from the provisional changes, then written -->
	<div class="epochs-section">
		<h3>Epochs</h3>
		{#if orgs.length === 0}
			<p class="muted">No Organisation is held, so there is no epoch to propose.</p>
		{:else}
			{#each orgs as o (o.org_id)}
				<p class="epoch-row">
					<span class="mono small">…{o.org_id.slice(-12)} (epoch {o.epoch})</span>
					<button onclick={() => onProposeNextEpoch?.(o.org_id)} disabled={epochBusy !== null}>Propose next epoch</button>
				</p>
			{/each}
		{/if}
		{#each proposedUpdates as update (update.org_id)}
			<p class="epoch-row">
				<span class="mono small">epoch {update.epoch} proposed for …{update.org_id.slice(-12)}</span>
				<button onclick={() => writeUpdate(update.org_id)} disabled={epochBusy !== null}>Write to chain</button>
			</p>
		{/each}
		{#if epochMessage}<p class="epoch-message">{epochMessage}</p>{/if}
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

	<!-- LLR-k5rf3r, LLR-md6hhz: discarded changes, kept until dismissed -->
	{#if notices.length > 0}
		<div class="notices-section">
			<h3>Changes left out of an adopted epoch</h3>
			{#each notices as notice (notice.id)}
				<div class="notice">
					<p>{notice.text}</p>
					<button onclick={() => proposeAgain(notice)}>Propose again</button>
					<button onclick={() => onDismissNotice?.(notice.id)}>Dismiss</button>
				</div>
			{/each}
			{#if confirming}
				<!-- LLR-e63yrs: the confirmation of REQ-88npkt, as for the first admission -->
				<div class="confirmation">
					<dl>
						<dt>Invited as</dt><dd>{confirming.confirmation.invitee_name}</dd>
						<dt>Request</dt><dd>{confirming.confirmation.name} {confirming.confirmation.surname}</dd>
						<dt>DevicePublicKey</dt><dd class="mono">{confirming.confirmation.device_public_key}</dd>
						<dt>Handle</dt><dd>{confirming.again.handle}</dd>
					</dl>
					{#if confirming.confirmation.mismatch}
						<p class="warn" role="alert">The name in the request is not the name you typed for this Invite.</p>
					{/if}
					<label class="confirm">
						<input type="checkbox" bind:checked={confirmedAgain} />
						This request is from the person I invited
					</label>
					<button onclick={confirmProposeAgain} disabled={!gateAgain.ok}>Propose admission again</button>
					<button onclick={cancelProposeAgain}>Cancel</button>
					{#if !gateAgain.ok}<p class="muted">{gateAgain.message}</p>{/if}
				</div>
			{/if}
			{#if reproposeErr}<p class="err">{reproposeErr}</p>{/if}
		</div>
	{/if}

	<!-- LLR-fsk4g2, LLR-9r6y7p: each Device's outcome of a send, or a refusal of the whole; no action but Dismiss -->
	{#if sendReports.length > 0}
		<div class="sends-section">
			<h3>Sends and requests</h3>
			{#each sendReports as report (report.id)}
				<div class="send-report">
					<p>{report.title}</p>
					{#if report.kind === 'sends'}
						{#if report.outcomes.length === 0}
							<p class="muted">No other Device to send to.</p>
						{:else}
							<ul class="mono small">
								{#each report.outcomes as outcome, position (position)}
									<li>…{outcome.device.slice(-16)}: {outcome.result === 'sent' ? 'sent' : `failed: ${outcome.reason}`}</li>
								{/each}
							</ul>
						{/if}
					{/if}
					<button onclick={() => onDismissReport?.(report.id)}>Dismiss</button>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Revoked orgs -->
	{#if revokedOrgs.length > 0}
		<div class="revoked-section">
			<h3>Revoked &amp; Self-Deleted</h3>
			{#each revokedOrgs as orgId, i (i)}
				<p class="revoked-entry">
					{orgId === null ? 'Removed from a provisional founding' : `Removed from org …${orgId.slice(-12)}`}
				</p>
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
	.notice { border-left: 3px solid #ff9800; padding-left: 0.5rem; }
	.confirmation { background: #0d0d1a; border: 1px solid #222; border-radius: 4px; padding: 0.75rem; margin-top: 0.5rem; }
	.warn { color: #ff9800; font-size: 0.85rem; }
</style>
