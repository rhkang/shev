<script lang="ts">
	import { onMount } from 'svelte';
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
		<div class="card stat-card">
			<div class="stat-label">Total Jobs</div>
			<div class="stat-value">{status.total_jobs}</div>
		</div>
		<div class="card stat-card">
			<div class="stat-label">Pending</div>
			<div class="stat-value pending">{status.pending_jobs}</div>
		</div>
		<div class="card stat-card">
			<div class="stat-label">Running</div>
			<div class="stat-value running">{status.running_jobs}</div>
		</div>
		<div class="card stat-card">
			<div class="stat-label">Completed</div>
			<div class="stat-value completed">{status.completed_jobs}</div>
		</div>
		<div class="card stat-card">
			<div class="stat-label">Failed</div>
			<div class="stat-value failed">{status.failed_jobs}</div>
		</div>
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
