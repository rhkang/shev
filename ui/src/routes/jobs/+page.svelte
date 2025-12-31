<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api, type Job, type JobStatus, JOB_STATUSES } from '$lib/api';

	let jobs = $state<Job[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let selectedStatuses = $state<Set<JobStatus>>(new Set(JOB_STATUSES));
	let dropdownOpen = $state(false);

	// Initialize from URL query params
	function initFromUrl() {
		const statusParam = $page.url.searchParams.get('status');
		if (statusParam) {
			const statuses = statusParam.split(',').filter((s): s is JobStatus =>
				JOB_STATUSES.includes(s as JobStatus)
			);
			if (statuses.length > 0) {
				selectedStatuses = new Set(statuses);
			}
		}
	}

	async function fetchJobs() {
		loading = true;
		try {
			jobs = await api.getJobs();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch jobs';
		} finally {
			loading = false;
		}
	}

	async function cancelJob(id: string) {
		try {
			await api.cancelJob(id);
			await fetchJobs();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to cancel job');
		}
	}

	function formatDate(date: string | null) {
		if (!date) return '-';
		return new Date(date).toLocaleString();
	}

	function toggleStatus(status: JobStatus) {
		const newSet = new Set(selectedStatuses);
		if (newSet.has(status)) {
			newSet.delete(status);
		} else {
			newSet.add(status);
		}
		selectedStatuses = newSet;
		updateUrl();
	}

	function selectAll() {
		selectedStatuses = new Set(JOB_STATUSES);
		updateUrl();
	}

	function selectNone() {
		selectedStatuses = new Set();
		updateUrl();
	}

	function updateUrl() {
		const url = new URL($page.url);
		if (selectedStatuses.size === JOB_STATUSES.length) {
			url.searchParams.delete('status');
		} else if (selectedStatuses.size > 0) {
			url.searchParams.set('status', Array.from(selectedStatuses).join(','));
		} else {
			url.searchParams.delete('status');
		}
		goto(url.toString(), { replaceState: true, noScroll: true });
	}

	function getStatusLabel(): string {
		if (selectedStatuses.size === JOB_STATUSES.length) return 'All Statuses';
		if (selectedStatuses.size === 0) return 'No Status';
		if (selectedStatuses.size === 1) return Array.from(selectedStatuses)[0];
		return `${selectedStatuses.size} selected`;
	}

	let filteredJobs = $derived(
		jobs.filter(job => selectedStatuses.has(job.status))
	);

	onMount(() => {
		initFromUrl();
		fetchJobs();
		const interval = setInterval(fetchJobs, 5000);
		return () => clearInterval(interval);
	});
</script>

<div class="header">
	<h2>Jobs</h2>
	<div class="controls">
		<div class="dropdown" class:open={dropdownOpen}>
			<button
				class="dropdown-toggle"
				onclick={() => dropdownOpen = !dropdownOpen}
				onblur={() => setTimeout(() => dropdownOpen = false, 150)}
			>
				{getStatusLabel()}
				<span class="arrow">▼</span>
			</button>
			{#if dropdownOpen}
				<div class="dropdown-menu">
					<div class="dropdown-actions">
						<button class="link-btn" onclick={selectAll}>All</button>
						<button class="link-btn" onclick={selectNone}>None</button>
					</div>
					{#each JOB_STATUSES as status}
						<label class="dropdown-item">
							<input
								type="checkbox"
								checked={selectedStatuses.has(status)}
								onchange={() => toggleStatus(status)}
							/>
							<span class="badge {status.toLowerCase()}">{status}</span>
						</label>
					{/each}
				</div>
			{/if}
		</div>
		<button class="secondary" onclick={fetchJobs}>Refresh</button>
	</div>
</div>

{#if loading && jobs.length === 0}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if filteredJobs.length === 0}
	<p class="muted">No jobs found</p>
{:else}
	<div class="card">
		<table>
			<thead>
				<tr>
					<th>ID</th>
					<th>Event Type</th>
					<th>Status</th>
					<th>Started</th>
					<th>Finished</th>
					<th>Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each filteredJobs as job}
					<tr>
						<td><a href="/jobs/{job.id}">{job.id.slice(0, 8)}...</a></td>
						<td>{job.event.event_type}</td>
						<td>
							<span class="badge {job.status.toLowerCase()}">{job.status}</span>
						</td>
						<td>{formatDate(job.started_at)}</td>
						<td>{formatDate(job.finished_at)}</td>
						<td>
							{#if job.status === 'Pending' || job.status === 'Running'}
								<button class="danger small" onclick={() => cancelJob(job.id)}>Cancel</button>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
	}

	.controls {
		display: flex;
		gap: 0.5rem;
	}

	.dropdown {
		position: relative;
	}

	.dropdown-toggle {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 4px;
		cursor: pointer;
		min-width: 140px;
		justify-content: space-between;
	}

	.dropdown-toggle:hover {
		border-color: var(--primary);
	}

	.arrow {
		font-size: 0.7rem;
		transition: transform 0.2s;
	}

	.dropdown.open .arrow {
		transform: rotate(180deg);
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 4px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
		z-index: 100;
		min-width: 160px;
	}

	.dropdown-actions {
		display: flex;
		gap: 0.5rem;
		padding: 0.5rem;
		border-bottom: 1px solid var(--border);
	}

	.link-btn {
		background: none;
		border: none;
		color: var(--primary);
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		font-size: 0.875rem;
	}

	.link-btn:hover {
		text-decoration: underline;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem;
		cursor: pointer;
	}

	.dropdown-item:hover {
		background: var(--bg-tertiary);
	}

	.dropdown-item input {
		width: auto;
		margin: 0;
	}

	.error {
		color: var(--error);
	}

	.muted {
		color: var(--text-muted);
	}

	button.small {
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
	}
</style>
