<script lang="ts">
	import StatusBar from '$lib/components/StatusBar.svelte';
	import PersonaList from '$lib/components/PersonaList.svelte';
	import CreateOrg from '$lib/components/CreateOrg.svelte';
	import Invite from '$lib/components/Invite.svelte';
	import Admit from '$lib/components/Admit.svelte';
	import Membership from '$lib/components/Membership.svelte';
	import Revoke from '$lib/components/Revoke.svelte';
	import { onMount } from 'svelte';
	import {
		verifyOrganisations,
		onVerificationResult,
		listAdmissionRequests,
		onAdmissionRequested,
		cancelInvite,
		proposeUpdatedOrganisation,
		writeOrganisationToChain,
		type VerificationEntryDto,
		type AdmissionRequestDto
	} from '$lib/api';
	import { fileVerificationResults } from '$lib/verification';
	import {
		applyDiscarded,
		applySendReports,
		dismissNotice,
		dismissSendReport,
		fileAdmissionRequests,
		withdrawAdmissionRequest,
		type AdmissionConfirmation,
		type DiscardedNotice,
		type SendReport
	} from '$lib/notices';
	import { afterWrite, fileProposal, proposalMessage, writeMessage, type ProposedUpdate } from '$lib/epoch';
	import { subscribeAll, type Unlisten } from '$lib/receiver';

	type TabId = 'personas' | 's1' | 's2' | 's3' | 's4' | 's5';

	let activeTab = $state<TabId>('personas');
	/** Increment to signal PersonaList to reload */
	let reloadToken = $state(0);
	/** LLR-84cxpt: every verification result shown, newest first. */
	let results = $state<VerificationEntryDto[]>([]);
	let startupError = $state('');

	function notifyReload() {
		reloadToken += 1;
	}

	/** LLR-k5rf3r, LLR-cgtav9: notices for discarded changes, kept here so a tab switch keeps them. */
	let notices = $state<DiscardedNotice[]>([]);
	/** LLR-9r6y7p, LLR-cgtav9: the send and request reports, kept here likewise. */
	let sendReports = $state<SendReport[]>([]);

	/**
	 * LLR-84cxpt, LLR-cgtav9: every verification entry the webview receives
	 * (at mount, on "Verify with chain", an included write's, each
	 * `verification-result` event) is filed here, into the results, the
	 * notices and the send reports alike.
	 */
	function fileResults(entries: VerificationEntryDto[]) {
		results = fileVerificationResults(results, entries);
		notices = entries.reduce(applyDiscarded, notices);
		sendReports = entries.reduce(applySendReports, sendReports);
	}

	function dismissDiscarded(id: string) {
		notices = dismissNotice(notices, id);
	}

	function dismissReport(id: string) {
		sendReports = dismissSendReport(sendReports, id);
	}

	/** LLR-tbtrb3: the updated Organisations proposed and not yet written, kept across tab switches. */
	let proposedUpdates = $state<ProposedUpdate[]>([]);
	/**
	 * LLR-5vgpkd, LLR-tbtrb3: what the Admin was last told of a proposal or a
	 * write, and the Organisation whose proposal or write is in flight. Kept
	 * here, so an outcome landing after a tab switch is shown and the guard
	 * outlives the panel.
	 */
	let epochMessage = $state('');
	let epochBusy = $state<string | null>(null);

	/** LLR-5vgpkd: "Propose next epoch" for `orgId`. */
	async function proposeNextEpoch(orgId: string) {
		if (epochBusy !== null) return;
		epochBusy = orgId;
		try {
			const proposed = await proposeUpdatedOrganisation(orgId);
			proposedUpdates = fileProposal(proposedUpdates, orgId, proposed);
			epochMessage = proposalMessage(orgId, { proposed });
		} catch (refusal) {
			epochMessage = proposalMessage(orgId, { refused: String(refusal) });
		} finally {
			epochBusy = null;
		}
	}

	/** LLR-tbtrb3, LLR-cgtav9: "Write to chain" for the update proposed for `orgId`; an included write's verification is filed as every verification is. */
	async function writeUpdate(orgId: string) {
		epochBusy = orgId;
		try {
			const written = await writeOrganisationToChain({ orgId });
			if (written.result === 'included') fileResults([written.verification]);
			proposedUpdates = afterWrite(proposedUpdates, orgId, written);
			epochMessage = writeMessage({ written });
		} catch (refusal) {
			epochMessage = writeMessage({ rejected: String(refusal) });
		} finally {
			epochBusy = null;
		}
		notifyReload();
	}

	/** LLR-5mw2dj, LLR-dhtec7: the kept admission requests, listed at start and announced on arrival. */
	let admissions = $state<AdmissionConfirmation[]>([]);
	/** LLR-5mw2dj: why listing the kept admission requests at start failed, apart from the verification. */
	let admissionsError = $state('');
	/** Why the listeners for verification results and admission requests did not start. */
	let listenError = $state('');

	function fileAdmissions(requests: AdmissionRequestDto[]) {
		admissions = fileAdmissionRequests(admissions, requests);
	}

	/** LLR-e7bbm5: what the Admin was told of the last cancel, and whether one is in flight, kept across tab switches. */
	let cancelMessage = $state('');
	let cancelBusy = $state(false);

	/** LLR-e7bbm5: "Cancel invite"; a cancelled Invite's request leaves the list every panel shows. True once cancelled. */
	async function cancelAdmission(confirmation: AdmissionConfirmation): Promise<boolean> {
		cancelBusy = true;
		try {
			await cancelInvite(confirmation.org_id, confirmation.invite_id);
			admissions = withdrawAdmissionRequest(admissions, confirmation.org_id, confirmation.invite_id);
			cancelMessage = `Invite cancelled: the admission request of ${confirmation.name} ${confirmation.surname} is removed.`;
			return true;
		} catch (refusal) {
			cancelMessage = String(refusal);
			return false;
		} finally {
			cancelBusy = false;
		}
	}

	/** LLR-rhc555: the founders of the foundings proposed on the creation form, newest first. */
	let proposedFounders = $state<string[]>([]);

	function fileProposedFounder(founder: string) {
		proposedFounders = [founder, ...proposedFounders];
	}

	onMount(() => {
		// REQ-rq8g2v: the cleanup cancels exactly the listeners registered.
		let torndown = false;
		let cleanup: Unlisten | null = null;
		// A refused subscription lists nothing (no listener started), so its
		// refusal is shown rather than left as an empty list of admission requests.
		subscribeAll([
			[onVerificationResult, (entry: VerificationEntryDto) => fileResults([entry])],
			[onAdmissionRequested, (request: AdmissionRequestDto) => fileAdmissions([request])]
		])
			.then((unlisten) => {
				if (torndown) unlisten();
				else cleanup = unlisten;
				// LLR-5mw2dj: listed once, after the listener started, so a request
				// arriving during the start is neither missed nor shown twice.
				listAdmissionRequests()
					.then(fileAdmissions)
					.catch((refusal) => {
						admissionsError = String(refusal);
					});
			})
			.catch((refusal) => {
				listenError = String(refusal);
			});
		// LLR-j2x9jb: one verification at start; the periodic one is the Rust half's.
		verifyOrganisations()
			.then(fileResults)
			.catch((e) => {
				startupError = String(e);
			})
			.finally(notifyReload);
		return () => {
			torndown = true;
			cleanup?.();
		};
	});

	const tabs: { id: TabId; label: string }[] = [
		{ id: 'personas', label: 'Personas / Orgs' },
		{ id: 's1', label: 'S1: Create Org' },
		{ id: 's2', label: 'S2: Invite' },
		{ id: 's3', label: 'S3: Admit' },
		{ id: 's4', label: 'S4: Membership' },
		{ id: 's5', label: 'S5: Revoke' }
	];
</script>

<div class="app-shell">
	<header class="app-header">
		<h1>ODS Phase 2 PoC</h1>
		<span class="subtitle">Organisational Data Sovereignty — five-story demo</span>
	</header>

	<StatusBar />

	<nav class="tab-bar">
		{#each tabs as t (t.id)}
			<button
				class="tab"
				class:active={activeTab === t.id}
				onclick={() => (activeTab = t.id)}
			>
				{t.label}
			</button>
		{/each}
	</nav>

	<div class="content">
		{#if activeTab === 'personas'}
			{#if startupError}<p class="err">verification at start failed: {startupError}</p>{/if}
			{#if listenError}<p class="err">listening for verification results and admission requests at start failed: {listenError}</p>{/if}
			{#if admissionsError}<p class="err">listing the admission requests at start failed: {admissionsError}</p>{/if}
			<PersonaList {reloadToken} {results} {fileResults} proposed={proposedFounders} />
		{:else if activeTab === 's1'}
			<CreateOrg {notifyReload} onProposed={fileProposedFounder} />
		{:else if activeTab === 's2'}
			<Invite {notifyReload} />
		{:else if activeTab === 's3'}
			<Admit {notifyReload} requests={admissions} {cancelMessage} {cancelBusy} onCancel={cancelAdmission} />
		{:else if activeTab === 's4'}
			<Membership
				{notifyReload}
				requests={admissions}
				{notices}
				{sendReports}
				onDismissNotice={dismissDiscarded}
				onDismissReport={dismissReport}
				{proposedUpdates}
				{epochMessage}
				{epochBusy}
				onProposeNextEpoch={proposeNextEpoch}
				onWriteUpdate={writeUpdate}
			/>
		{:else if activeTab === 's5'}
			<Revoke {notifyReload} />
		{/if}
	</div>
</div>

<style>
	:global(*, *::before, *::after) { box-sizing: border-box; }
	:global(body) {
		margin: 0;
		background: #0a0a0a;
		color: #ccc;
		font-family: system-ui, -apple-system, sans-serif;
	}

	.app-shell {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	.app-header {
		background: #0f0f1e;
		border-bottom: 1px solid #222;
		padding: 0.6rem 1rem;
		display: flex;
		align-items: baseline;
		gap: 0.75rem;
	}
	h1 { margin: 0; font-size: 1.1rem; color: #7ec8e3; font-weight: 600; }
	.subtitle { color: #555; font-size: 0.8rem; }

	.tab-bar {
		display: flex;
		gap: 0;
		background: #111;
		border-bottom: 1px solid #222;
		overflow-x: auto;
	}

	.tab {
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		color: #777;
		cursor: pointer;
		font-size: 0.82rem;
		padding: 0.5rem 0.9rem;
		white-space: nowrap;
		transition: color 0.1s, border-color 0.1s;
	}
	.tab:hover { color: #bbb; background: #181818; }
	.tab.active { color: #7ec8e3; border-bottom-color: #7ec8e3; background: #0d0d18; }

	.err { color: #f44336; font-size: 0.85rem; }

	.content {
		flex: 1;
		padding: 1rem;
		max-width: 900px;
	}
</style>
