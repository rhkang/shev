<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { api, type Job } from '$lib/api';

	let job = $state<Job | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function fetchJob() {
		const id = $page.params.id;
		if (!id) return;
		try {
			job = await api.getJob(id);
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch job';
		} finally {
			loading = false;
		}
	}

	async function cancelJob() {
		if (!job) return;
		try {
			await api.cancelJob(job.id);
			await fetchJob();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to cancel job');
		}
	}

	function formatDate(date: string | null) {
		if (!date) return '-';
		return new Date(date).toLocaleString();
	}

	onMount(() => {
		fetchJob();
		const interval = setInterval(fetchJob, 3000);
		return () => clearInterval(interval);
	});
</script>

<div class="header">
	<a href="/jobs" class="back">&larr; Back to Jobs</a>
	<h2>Job Details</h2>
</div>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if job}
	<div class="card">
		<div class="detail-grid">
			<div class="detail">
				<span class="label">ID</span>
				<span class="value">{job.id}</span>
			</div>
			<div class="detail">
				<span class="label">Status</span>
				<span class="value">
					<span class="badge {job.status.toLowerCase()}">{job.status}</span>
				</span>
			</div>
			<div class="detail">
				<span class="label">Event Type</span>
				<span class="value">{job.event.event_type}</span>
			</div>
			<div class="detail">
				<span class="label">Handler ID</span>
				<span class="value">{job.handler_id}</span>
			</div>
			<div class="detail">
				<span class="label">Event Time</span>
				<span class="value">{formatDate(job.event.timestamp)}</span>
			</div>
			<div class="detail">
				<span class="label">Started At</span>
				<span class="value">{formatDate(job.started_at)}</span>
			</div>
			<div class="detail">
				<span class="label">Finished At</span>
				<span class="value">{formatDate(job.finished_at)}</span>
			</div>
		</div>

		{#if job.status === 'Pending' || job.status === 'Running'}
			<div class="actions">
				<button class="danger" onclick={cancelJob}>Cancel Job</button>
			</div>
		{/if}
	</div>

	{#if job.event.context}
		<div class="card context-card">
			<h3>Context</h3>
			<pre>{job.event.context}</pre>
		</div>
	{/if}

	{#if job.output}
		<div class="card output-card">
			<h3>Output</h3>
			<pre>{job.output}</pre>
		</div>
	{/if}

	{#if job.error}
		<div class="card error-card">
			<h3>Error</h3>
			<pre>{job.error}</pre>
		</div>
	{/if}
{/if}

<style>
	.header {
		margin-bottom: 1.5rem;
	}

	.back {
		display: inline-block;
		margin-bottom: 0.5rem;
		color: var(--text-muted);
	}

	.error {
		color: var(--error);
	}

	.detail-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
	}

	.detail {
		display: flex;
		flex-direction: column;
	}

	.label {
		color: var(--text-muted);
		font-size: 0.875rem;
		margin-bottom: 0.25rem;
	}

	.value {
		word-break: break-all;
	}

	.actions {
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--border);
	}

	.context-card, .output-card, .error-card {
		margin-top: 1rem;
	}

	.context-card h3, .output-card h3, .error-card h3 {
		margin-bottom: 0.75rem;
		font-size: 1rem;
	}

	.error-card pre {
		color: var(--error);
	}
</style>
