<script lang="ts">
	/**
	 * Story 2: Invite flow (two sides).
	 * A (Member):  export_invite (REQ-prjja8) → show/copy the Invite Blob.
	 * B (Invitee): paste an Invite → import_invite (REQ-yazum3) → choose a
	 *              Persona → read REPLY_WARNING and confirm (REQ-ab2mfz,
	 *              RC-wzb48r) → produce_invite_reply (REQ-tcutr6) → copy.
	 *              Only Personas bound to no Organisation are offered
	 *              (LLR-rt8gdz).
	 */
	import {
		listOrgs,
		listPersonas,
		exportInvite,
		importInvite,
		createPersona,
		produceInviteReply,
		type OrgDto,
		type PersonaDto,
		type InviteDto
	} from '$lib/api';
	import { REPLY_WARNING, replyGate, unboundPersonas } from '$lib/invite';

	interface Props {
		notifyReload?: () => void;
	}
	let { notifyReload }: Props = $props();

	// --- Side A: export invite ---
	let orgs = $state<OrgDto[]>([]);
	let selectedOrgId = $state('');
	let orgName = $state('');
	let inviteeName = $state('');
	let inviteBlob = $state('');
	let exportBusy = $state(false);
	let exportErr = $state('');

	async function loadOrgs() {
		try {
			orgs = await listOrgs();
			if (orgs.length > 0 && !selectedOrgId) selectedOrgId = orgs[0].org_id;
		} catch (e) {
			exportErr = String(e);
		}
	}

	$effect(() => { loadOrgs(); });

	async function doExportInvite() {
		if (!selectedOrgId) { exportErr = 'Select an org.'; return; }
		exportBusy = true;
		exportErr = '';
		try {
			inviteBlob = await exportInvite(selectedOrgId, orgName.trim(), inviteeName.trim());
		} catch (e) {
			exportErr = String(e);
		} finally {
			exportBusy = false;
		}
	}

	// --- Side B: import invite → choose Persona → confirm → produce reply ---
	let pastedInvite = $state('');
	let imported = $state<InviteDto | null>(null);
	let importBusy = $state(false);
	let importErr = $state('');

	let personas = $state<PersonaDto[]>([]);
	let personaId = $state('');
	let confirmed = $state(false);
	let replyBlob = $state('');
	let replyBusy = $state(false);
	let replyErr = $state('');

	let gate = $derived(replyGate({ personaId, confirmed }));

	// Persona form (create one to reply as)
	let newHandle = $state('');
	let newName = $state('');
	let newSurname = $state('');
	let createBusy = $state(false);
	let createErr = $state('');

	async function loadPersonas() {
		try {
			// LLR-rt8gdz: a Persona bound to an Organisation is not offered.
			personas = unboundPersonas(await listPersonas());
		} catch (e) {
			replyErr = String(e);
		}
	}

	async function doImportInvite() {
		if (!pastedInvite.trim()) { importErr = 'Paste an Invite first.'; return; }
		importBusy = true;
		importErr = '';
		imported = null;
		confirmed = false;
		replyBlob = '';
		replyErr = '';
		try {
			imported = await importInvite(pastedInvite.trim());
			await loadPersonas();
		} catch (e) {
			importErr = String(e);
		} finally {
			importBusy = false;
		}
	}

	async function doCreatePersona() {
		if (!newHandle.trim() || !newName.trim()) { createErr = 'Handle and name required.'; return; }
		createBusy = true;
		createErr = '';
		try {
			personaId = await createPersona(newHandle.trim(), newName.trim(), newSurname.trim());
			await loadPersonas();
			notifyReload?.();
		} catch (e) {
			createErr = String(e);
		} finally {
			createBusy = false;
		}
	}

	async function doProduceReply() {
		if (!gate.ok) { replyErr = gate.message; return; }
		replyBusy = true;
		replyErr = '';
		try {
			replyBlob = await produceInviteReply(pastedInvite.trim(), personaId, confirmed);
		} catch (e) {
			replyErr = String(e);
		} finally {
			replyBusy = false;
		}
	}

	function copy(text: string) {
		navigator.clipboard.writeText(text);
	}
</script>

<section class="panel">
	<h2>Story 2: Invite Flow</h2>

	<div class="two-col">
		<!-- Side A: a Member exports an Invite -->
		<div class="side">
			<h3>A — Export Invite (Member)</h3>
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
					Organisation name (as you want the invitee to see it)
					<input type="text" bind:value={orgName} placeholder="Acme Co-op" disabled={exportBusy} />
				</label>
				<label>
					Invitee's full name (your guess)
					<input type="text" bind:value={inviteeName} placeholder="Bob Builder" disabled={exportBusy} />
				</label>
				<button onclick={doExportInvite} disabled={exportBusy}>
					{exportBusy ? 'Exporting…' : 'Export Invite'}
				</button>
				{#if exportErr}<p class="err">{exportErr}</p>{/if}
				{#if inviteBlob}
					<label>
						Invite (copy to the invitee)
						<textarea readonly rows="4" value={inviteBlob}></textarea>
					</label>
					<button onclick={() => copy(inviteBlob)}>Copy</button>
				{/if}
			{/if}
		</div>

		<!-- Side B: the invitee imports the Invite and replies -->
		<div class="side">
			<h3>B — Import Invite + Reply (Invitee)</h3>
			<label>
				Paste Invite
				<textarea rows="4" bind:value={pastedInvite} placeholder="Paste the Invite here…"></textarea>
			</label>
			<button onclick={doImportInvite} disabled={importBusy}>
				{importBusy ? 'Importing…' : 'Import Invite'}
			</button>
			{#if importErr}<p class="err">{importErr}</p>{/if}
			{#if imported}
				<dl>
					<dt>Organisation name</dt>
					<dd>{imported.org_name} <span class="warn">(stated by the sender, unverified)</span></dd>
					<dt>Org id</dt><dd class="mono">{imported.org_id}</dd>
					<dt>Invitee name</dt><dd>{imported.invitee_name}</dd>
				</dl>

				<label>
					Reply as Persona
					<select bind:value={personaId} disabled={replyBusy}>
						<option value="">— choose a Persona —</option>
						{#each personas as p (p.persona_id)}
							<option value={p.persona_id}>{p.handle} ({p.name} {p.surname})</option>
						{/each}
					</select>
				</label>

				<h4>Or create a Persona</h4>
				<label>Handle <input type="text" bind:value={newHandle} placeholder="bob" disabled={createBusy} /></label>
				<label>Name <input type="text" bind:value={newName} placeholder="Bob" disabled={createBusy} /></label>
				<label>Surname <input type="text" bind:value={newSurname} placeholder="Builder" disabled={createBusy} /></label>
				<button onclick={doCreatePersona} disabled={createBusy}>
					{createBusy ? 'Creating…' : 'Create Persona'}
				</button>
				{#if createErr}<p class="err">{createErr}</p>{/if}

				<div class="warning-box" role="alert">
					<p>{REPLY_WARNING}</p>
					<label class="confirm">
						<input type="checkbox" bind:checked={confirmed} disabled={replyBusy} />
						I understand
					</label>
				</div>
				<button onclick={doProduceReply} disabled={replyBusy || !gate.ok}>
					{replyBusy ? 'Producing…' : 'Produce reply'}
				</button>
				{#if !gate.ok}<p class="muted">{gate.message}</p>{/if}
				{#if replyErr}<p class="err">{replyErr}</p>{/if}
				{#if replyBlob}
					<label>
						Invite reply (send to the Invite's sender)
						<textarea readonly rows="4" value={replyBlob}></textarea>
					</label>
					<button onclick={() => copy(replyBlob)}>Copy</button>
				{/if}
			{/if}
		</div>
	</div>
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	h3 { font-size: 0.9rem; color: #aaa; margin: 0 0 0.5rem; }
	h4 { font-size: 0.85rem; color: #888; margin: 0.5rem 0 0.25rem; }
	.two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
	@media (max-width: 700px) { .two-col { grid-template-columns: 1fr; } }
	.side { display: flex; flex-direction: column; gap: 0.4rem; }
	label { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.82rem; color: #888; }
	label.confirm { flex-direction: row; align-items: center; gap: 0.4rem; color: #ddd; }
	input, select, textarea { background: #1a1a1a; border: 1px solid #333; color: #eee; padding: 0.3rem 0.5rem; border-radius: 3px; font-size: 0.82rem; font-family: monospace; resize: vertical; }
	input:disabled { opacity: 0.5; }
	button { cursor: pointer; background: #1a3a5c; color: #7ec8e3; border: 1px solid #2a5f8c; border-radius: 3px; padding: 0.3rem 0.75rem; font-size: 0.82rem; align-self: flex-start; }
	button:disabled { opacity: 0.5; cursor: default; }
	button:not(:disabled):hover { background: #204d7a; }
	dl { display: grid; grid-template-columns: 8rem 1fr; gap: 0.2rem 0.5rem; font-size: 0.82rem; margin: 0; }
	dt { color: #666; }
	dd { color: #ccc; margin: 0; word-break: break-all; }
	.mono { font-family: monospace; }
	.warning-box { background: #2a1f00; border: 1px solid #ff9800; border-radius: 4px; padding: 0.6rem; margin-top: 0.5rem; }
	.warning-box p { margin: 0 0 0.4rem; font-size: 0.82rem; color: #ffcc80; }
	.err { color: #f44336; font-size: 0.8rem; }
	.warn { color: #ff9800; font-size: 0.78rem; }
	.muted { color: #555; font-size: 0.82rem; }
</style>
