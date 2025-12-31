<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, type Status } from '$lib/api';

	let status = $state<Status | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	async function fetchStatus() {
		try {
			status = await api.getStatus();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch status';
		} finally {
			loading = false;
		}
	}

	async function reload() {
		loading = true;
		try {
			const result = await api.reload();
			await fetchStatus();
			alert(`Reloaded: ${result.handlers_loaded} handlers, ${result.timers_loaded} timers, ${result.schedules_loaded} schedules`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Reload failed';
		} finally {
			loading = false;
		}
	}

	function goToJobs(statusFilter?: string) {
		if (statusFilter) {
			goto(`/jobs?status=${statusFilter}`);
		} else {
			goto('/jobs');
		}
	}

	onMount(() => {
		fetchStatus();
		const interval = setInterval(fetchStatus, 5000);
		return () => clearInterval(interval);
	});
</script>

<h2>Dashboard</h2>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if status}
	<div class="stats-grid">
		<button class="card stat-card clickable" onclick={() => goToJobs()}>
			<div class="stat-label">Total Jobs</div>
			<div class="stat-value">{status.total_jobs}</div>
		</button>
		<button class="card stat-card clickable" onclick={() => goToJobs('Pending')}>
			<div class="stat-label">Pending</div>
			<div class="stat-value pending">{status.pending_jobs}</div>
		</button>
		<button class="card stat-card clickable" onclick={() => goToJobs('Running')}>
			<div class="stat-label">Running</div>
			<div class="stat-value running">{status.running_jobs}</div>
		</button>
		<button class="card stat-card clickable" onclick={() => goToJobs('Completed')}>
			<div class="stat-label">Completed</div>
			<div class="stat-value completed">{status.completed_jobs}</div>
		</button>
		<button class="card stat-card clickable" onclick={() => goToJobs('Failed')}>
			<div class="stat-label">Failed</div>
			<div class="stat-value failed">{status.failed_jobs}</div>
		</button>
	</div>

	<div class="actions">
		<button onclick={reload} disabled={loading}>
			Reload Config
		</button>
		<button class="secondary" onclick={fetchStatus}>
			Refresh
		</button>
	</div>
{/if}

<style>
	h2 {
		margin-bottom: 1.5rem;
	}

	.error {
		color: var(--error);
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.stat-card {
		text-align: center;
	}

	.stat-card.clickable {
		cursor: pointer;
		transition: transform 0.15s, box-shadow 0.15s;
		border: 1px solid var(--border);
	}

	.stat-card.clickable:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
		border-color: var(--primary);
	}

	.stat-label {
		color: var(--text-muted);
		font-size: 0.875rem;
		margin-bottom: 0.5rem;
	}

	.stat-value {
		font-size: 1.5rem;
		font-weight: 600;
	}

	.stat-value.pending { color: var(--warning); }
	.stat-value.running { color: var(--primary); }
	.stat-value.completed { color: var(--success); }
	.stat-value.failed { color: var(--error); }

	.actions {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
	}
</style>
