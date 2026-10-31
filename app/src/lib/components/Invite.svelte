<script lang="ts">
	/**
	 * Story 2: Invite flow (two sides).
	 * A (Admin):   issue_invite (LLR-y3zykm) → show/copy the Invite Blob.
	 * B (Invitee): paste an Invite → read_invite (LLR-b7ywy4) → name, surname
	 *              and preferred Handles → propose_member (a new Persona,
	 *              LLR-rg9vc7) → read ADMISSION_REQUEST_WARNING and confirm
	 *              (LLR-qcg6bz) → request_admission. A retry of a confirmed
	 *              request asks nothing again (LLR-prh433).
	 */
	import {
		listOrgs,
		issueInvite,
		readInvite,
		proposeMember,
		requestAdmission,
		type OrgDto,
		type InviteDto,
		type AdmissionDto
	} from '$lib/api';
	import { ADMISSION_REQUEST_WARNING, admissionRequestGate } from '$lib/invite';
	import { admissionRequestMessage, type RequestMessage } from '$lib/notices';

	interface Props {
		notifyReload?: () => void;
	}
	let { notifyReload }: Props = $props();

	// --- Side A: issue an Invite ---
	let orgs = $state<OrgDto[]>([]);
	let selectedOrgId = $state('');
	let inviteeName = $state('');
	let inviteBlob = $state('');
	let issueBusy = $state(false);
	let issueErr = $state('');

	async function loadOrgs() {
		try {
			orgs = await listOrgs();
			if (orgs.length > 0 && !selectedOrgId) selectedOrgId = orgs[0].org_id;
		} catch (refusal) {
			issueErr = String(refusal);
		}
	}

	$effect(() => {
		loadOrgs();
	});

	async function doIssueInvite() {
		if (!selectedOrgId) {
			issueErr = 'Select an org.';
			return;
		}
		issueBusy = true;
		issueErr = '';
		try {
			inviteBlob = await issueInvite(selectedOrgId, inviteeName.trim());
		} catch (refusal) {
			issueErr = String(refusal);
		} finally {
			issueBusy = false;
		}
	}

	// --- Side B: read the Invite → propose a Member → confirm → request admission ---
	let pastedInvite = $state('');
	let read = $state<InviteDto | null>(null);
	let readBusy = $state(false);
	let readErr = $state('');

	let name = $state('');
	let surname = $state('');
	let handles = $state('');
	let personaId = $state('');
	let proposeBusy = $state(false);
	let proposeErr = $state('');

	/** LLR-prh433: the provisional Members whose request the user confirmed. */
	let confirmedPersonas = $state<string[]>([]);
	let confirmed = $state(false);
	let confirmedBefore = $derived(personaId !== '' && confirmedPersonas.includes(personaId));
	let gate = $derived(admissionRequestGate({ confirmedBefore, confirmed }));
	let requestBusy = $state(false);
	let requestErr = $state('');
	let outcome = $state<AdmissionDto | null>(null);
	/** LLR-d8998k: the error shown above the form, for an invite another Device used. */
	let requestMessage = $state<RequestMessage | null>(null);

	async function doReadInvite() {
		if (!pastedInvite.trim()) {
			readErr = 'Paste an Invite first.';
			return;
		}
		readBusy = true;
		readErr = '';
		read = null;
		personaId = '';
		confirmed = false;
		outcome = null;
		try {
			read = await readInvite(pastedInvite.trim());
		} catch (refusal) {
			readErr = String(refusal);
		} finally {
			readBusy = false;
		}
	}

	async function doProposeMember() {
		const preferred = handles
			.split(',')
			.map((preferredHandle) => preferredHandle.trim())
			.filter((preferredHandle) => preferredHandle !== '');
		proposeBusy = true;
		proposeErr = '';
		try {
			personaId = (await proposeMember(pastedInvite.trim(), name.trim(), surname.trim(), preferred)).persona_id;
			notifyReload?.();
		} catch (refusal) {
			proposeErr = String(refusal);
		} finally {
			proposeBusy = false;
		}
	}

	async function doRequestAdmission() {
		requestMessage = null;
		if (!gate.ok) {
			requestErr = gate.message;
			return;
		}
		requestBusy = true;
		requestErr = '';
		try {
			outcome = await requestAdmission(personaId, true);
			requestMessage = admissionRequestMessage(outcome);
			if (!confirmedPersonas.includes(personaId)) confirmedPersonas = [...confirmedPersonas, personaId];
		} catch (refusal) {
			requestErr = String(refusal);
		} finally {
			requestBusy = false;
		}
	}

	function outcomeText(requestOutcome: AdmissionDto): string {
		switch (requestOutcome.result) {
			case 'delivered':
				return 'Delivered: the Admin will see your request.';
			case 'already_received':
				return 'The Admin already has this request.';
			case 'invite_used_by_another_device':
				return 'Another Device already used this invite.';
			case 'refused':
				return 'The receiving Device refused the request.';
			case 'not_delivered':
				return `Not delivered: ${requestOutcome.reason}`;
		}
	}

	function copy(text: string) {
		navigator.clipboard.writeText(text);
	}
</script>

<section class="panel">
	<h2>Story 2: Invite Flow</h2>

	<div class="two-col">
		<div class="side">
			<h3>A — Issue Invite (Admin)</h3>
			{#if orgs.length === 0}
				<p class="muted">No orgs yet (create one in Story 1).</p>
			{:else}
				<label>
					Org
					<select bind:value={selectedOrgId}>
						{#each orgs as o (o.org_id)}
							<option value={o.org_id}>…{o.org_id.slice(-12)} (epoch {o.epoch})</option>
						{/each}
					</select>
				</label>
				<label>
					Invitee's full name
					<input type="text" bind:value={inviteeName} placeholder="Bob Builder" disabled={issueBusy} />
				</label>
				<button onclick={doIssueInvite} disabled={issueBusy}>{issueBusy ? 'Issuing…' : 'Issue Invite'}</button>
				{#if issueErr}<p class="err">{issueErr}</p>{/if}
				{#if inviteBlob}
					<label>
						Invite (copy to the invitee)
						<textarea readonly rows="4" value={inviteBlob}></textarea>
					</label>
					<button onclick={() => copy(inviteBlob)}>Copy</button>
				{/if}
			{/if}
		</div>

		<div class="side">
			<h3>B — Read Invite + Request Admission (Invitee)</h3>
			{#if requestMessage}<p class="err prominent" role="alert">{requestMessage.text}</p>{/if}
			<label>
				Paste Invite
				<textarea rows="4" bind:value={pastedInvite} placeholder="Paste the Invite here…"></textarea>
			</label>
			<button onclick={doReadInvite} disabled={readBusy}>{readBusy ? 'Reading…' : 'Read Invite'}</button>
			{#if readErr}<p class="err">{readErr}</p>{/if}
			{#if read}
				<dl>
					<dt>Organisation name</dt>
					<dd>{read.org_name} <span class="warn">(stated by the sender, unverified)</span></dd>
					<dt>Org id</dt><dd class="mono">{read.org_id}</dd>
					<dt>Invitee name</dt><dd>{read.invitee_name}</dd>
					<dt>Issued by Device</dt><dd class="mono">{read.issuer_device_key}</dd>
				</dl>
				{#if personaId === ''}
					<label>Name <input type="text" bind:value={name} disabled={proposeBusy} /></label>
					<label>Surname <input type="text" bind:value={surname} disabled={proposeBusy} /></label>
					<label>Preferred Handles (1 to 3, comma-separated) <input type="text" bind:value={handles} placeholder="bob, builder" disabled={proposeBusy} /></label>
					<button onclick={doProposeMember} disabled={proposeBusy}>{proposeBusy ? 'Proposing…' : 'Propose Member'}</button>
					{#if proposeErr}<p class="err">{proposeErr}</p>{/if}
				{:else}
					{#if !confirmedBefore}
						<div class="warning-box" role="alert">
							<p>{ADMISSION_REQUEST_WARNING}</p>
							<label class="confirm">
								<input type="checkbox" bind:checked={confirmed} disabled={requestBusy} />
								I understand
							</label>
						</div>
					{/if}
					<button onclick={doRequestAdmission} disabled={requestBusy || !gate.ok}>
						{requestBusy ? 'Requesting…' : confirmedBefore ? 'Request admission again' : 'Request admission'}
					</button>
					{#if !gate.ok}<p class="muted">{gate.message}</p>{/if}
					{#if requestErr}<p class="err">{requestErr}</p>{/if}
					{#if outcome}<p class="small">{outcomeText(outcome)}</p>{/if}
				{/if}
			{/if}
		</div>
	</div>
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	h3 { font-size: 0.9rem; color: #aaa; margin: 0 0 0.5rem; }
	.two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
	@media (max-width: 700px) { .two-col { grid-template-columns: 1fr; } }
	.side { display: flex; flex-direction: column; gap: 0.4rem; }
	label { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.82rem; color: #888; }
	label.confirm { flex-direction: row; align-items: center; gap: 0.4rem; color: #ddd; }
	input, select, textarea { background: #1a1a1a; border: 1px solid #333; color: #eee; padding: 0.3rem 0.5rem; border-radius: 3px; font-size: 0.82rem; font-family: monospace; resize: vertical; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.3rem 0.75rem; font-size: 0.82rem; align-self: flex-start; }
	button:disabled { opacity: 0.5; cursor: default; }
	dl { display: grid; grid-template-columns: 8rem 1fr; gap: 0.2rem 0.5rem; font-size: 0.82rem; margin: 0; }
	dt { color: #666; }
	dd { color: #ccc; margin: 0; word-break: break-all; }
	.warning-box { border: 1px solid #ff9800; border-radius: 4px; padding: 0.5rem; color: #ffcc80; font-size: 0.82rem; }
	.mono { font-family: monospace; }
	.small { font-size: 0.8rem; color: #aaa; }
	.err { color: #f44336; font-size: 0.8rem; }
	.prominent { font-size: 0.95rem; font-weight: bold; border: 1px solid #f44336; padding: 0.4rem; border-radius: 3px; }
	.warn { color: #ff9800; }
	.muted { color: #555; font-size: 0.8rem; }
</style>
