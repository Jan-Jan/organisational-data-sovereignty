<script lang="ts">
	import {
		listPersonas,
		listOrgs,
		verifyOrganisations,
		writeOrganisationToChain,
		type PersonaDto,
		type OrgDto,
		type VerificationEntryDto
	} from '$lib/api';
	import { verificationLine, verificationLabel, foundingWrites } from '$lib/verification';
	import { handleLabel } from '$lib/personas';

	interface Props {
		/** Increment this to trigger a reload */
		reloadToken?: number;
		/** LLR-84cxpt: every verification result the page filed, newest first. */
		results?: VerificationEntryDto[];
		/** Files the entries of a verification the user asked for into that list. */
		fileResults?: (entries: VerificationEntryDto[]) => void;
		/** LLR-rhc555: the founders of the foundings proposed here (`proposeFoundingOrganisation`'s `founder`). */
		proposed?: string[];
	}

	let { reloadToken = 0, results = [], fileResults, proposed = [] }: Props = $props();

	let verifying = $state(false);
	/** The founder whose founding is being written, if any. */
	let writing = $state<string | null>(null);

	/** LLR-j2x9jb: the "Verify with chain" button. */
	async function verify() {
		verifying = true;
		try {
			const entries = await verifyOrganisations();
			fileResults?.(entries);
			error = '';
		} catch (refusal) {
			error = String(refusal);
		} finally {
			verifying = false;
		}
		await reload();
	}

	/** LLR-rhc555: a founding's "Write to chain" button, by its founder's Device key. */
	async function writeFounding(founder: string) {
		writing = founder;
		try {
			const written = await writeOrganisationToChain({ founder });
			if (written.result === 'included') {
				fileResults?.([written.verification]);
				error = '';
			} else {
				error = `write ${written.result}: ${written.reason}`;
			}
		} catch (refusal) {
			error = String(refusal);
		} finally {
			writing = null;
		}
		await reload();
	}

	let personas = $state<PersonaDto[]>([]);
	let orgs = $state<OrgDto[]>([]);
	let error = $state('');
	let loading = $state(false);

	async function reload() {
		loading = true;
		try {
			[personas, orgs] = await Promise.all([listPersonas(), listOrgs()]);
			error = '';
		} catch (refusal) {
			error = String(refusal);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		// Re-run whenever reloadToken changes (dependency tracked by Svelte 5 runes)
		reloadToken;
		reload();
	});

	function orgLabel(orgId: string | null): string {
		if (!orgId) return '(unattached)';
		const o = orgs.find((x) => x.org_id === orgId);
		if (o) return `org …${orgId.slice(-8)} epoch=${o.epoch} members=${o.member_count}`;
		return `org …${orgId.slice(-8)}`;
	}

	function statusClass(status: string): string {
		if (status.includes('Active')) return 'active';
		if (status.includes('Revoked')) return 'revoked';
		return 'proposed';
	}
</script>

<section class="panel">
	<h2>Personas &amp; Orgs</h2>
	{#if loading}<p class="muted">loading…</p>{/if}
	{#if error}<p class="err">{error}</p>{/if}

	{#if personas.length === 0 && !loading}
		<p class="muted">No personas yet. Use "Story 1" to create one.</p>
	{:else}
		<table>
			<thead>
				<tr>
					<th>Handle</th>
					<th>Name</th>
					<th>Status</th>
					<th>Org</th>
					<th>Persona ID</th>
				</tr>
			</thead>
			<tbody>
				{#each personas as p (p.persona_id)}
					<tr>
						<td><strong>{handleLabel(p.handle)}</strong></td>
						<td>{p.name} {p.surname}</td>
						<td><span class="badge {statusClass(p.status)}">{p.status}</span></td>
						<td class="mono small">{orgLabel(p.org_id)}</td>
						<td class="mono small">…{p.persona_id.slice(-12)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}

	{#if orgs.length > 0}
		<h3>Orgs</h3>
		<table>
			<thead>
				<tr><th>Org ID</th><th>Epoch</th><th>Members</th><th>Root hash</th><th>Chain</th></tr>
			</thead>
			<tbody>
				{#each orgs as o (o.org_id)}
					<tr>
						<td class="mono small">…{o.org_id.slice(-12)}</td>
						<td>{o.epoch}</td>
						<td>{o.member_count}</td>
						<td class="mono small">…{o.root_hash.slice(-16)}</td>
						<td class:unverified={o.verification.status !== 'in_step' && o.verification.status !== 'adopted'}>{verificationLabel(o.verification)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}

	{#if results.length > 0}
		<h3>Last verifications</h3>
		<ul class="mono small">
			{#each results as result, index (index)}
				<li>{verificationLine(result)}</li>
			{/each}
		</ul>
	{/if}

	{#each foundingWrites(results, proposed) as founder (founder)}
		<p class="mono small">
			founding by {founder}
			<button onclick={() => writeFounding(founder)} disabled={writing !== null}>{writing === founder ? 'Writing…' : 'Write to chain'}</button>
		</p>
	{/each}

	<button onclick={reload}>Refresh</button>
	<button onclick={verify} disabled={verifying}>{verifying ? 'Verifying…' : 'Verify with chain'}</button>
</section>

<style>
	.panel { background: #111; padding: 1rem; border-radius: 6px; margin-bottom: 1rem; }
	h2 { margin-top: 0; font-size: 1rem; color: #ddd; }
	h3 { font-size: 0.9rem; color: #bbb; margin-top: 1rem; }
	table { border-collapse: collapse; width: 100%; font-size: 0.85rem; margin-bottom: 0.5rem; }
	th, td { text-align: left; padding: 0.25rem 0.5rem; border-bottom: 1px solid #2a2a2a; }
	th { color: #888; font-weight: normal; }
	td { color: #ccc; }
	.mono { font-family: monospace; }
	.small { font-size: 0.75rem; color: #777; }
	.badge { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; font-weight: bold; }
	.active { background: #1b4332; color: #4caf50; }
	.revoked { background: #4a1010; color: #f44336; }
	.proposed { background: #2a2a10; color: #ff9800; }
	.err { color: #f44336; font-size: 0.85rem; }
	.unverified { color: #ff9800; }
	.muted { color: #555; font-size: 0.85rem; }
	button { margin-top: 0.5rem; cursor: pointer; background: #222; color: #aaa; border: 1px solid #444; border-radius: 3px; padding: 0.25rem 0.75rem; font-size: 0.8rem; }
	button:hover { background: #333; }
</style>
