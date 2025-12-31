<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Job } from '$lib/api';

	let jobs = $state<Job[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let filter = $state<'all' | 'completed'>('all');

	async function fetchJobs() {
		loading = true;
		try {
			jobs = filter === 'all' ? await api.getJobs() : await api.getCompletedJobs();
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

	onMount(() => {
		fetchJobs();
		const interval = setInterval(fetchJobs, 5000);
		return () => clearInterval(interval);
	});

	$effect(() => {
		filter;
		fetchJobs();
	});
</script>

<div class="header">
	<h2>Jobs</h2>
	<div class="controls">
		<select bind:value={filter}>
			<option value="all">All Jobs</option>
			<option value="completed">Completed Only</option>
		</select>
		<button class="secondary" onclick={fetchJobs}>Refresh</button>
	</div>
</div>

{#if loading && jobs.length === 0}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if jobs.length === 0}
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
				{#each jobs as job}
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

	.controls select {
		width: auto;
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
