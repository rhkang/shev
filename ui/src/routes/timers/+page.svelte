<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Timer } from '$lib/api';

	let timers = $state<Timer[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function fetchTimers() {
		try {
			timers = await api.getTimers();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch timers';
		} finally {
			loading = false;
		}
	}

	function formatInterval(secs: number): string {
		if (secs < 60) return `${secs}s`;
		if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
		const hours = Math.floor(secs / 3600);
		const mins = Math.floor((secs % 3600) / 60);
		return `${hours}h ${mins}m`;
	}

	onMount(fetchTimers);
</script>

<div class="header">
	<h2>Timers</h2>
	<button class="secondary" onclick={fetchTimers}>Refresh</button>
</div>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if timers.length === 0}
	<p class="muted">No timers configured</p>
{:else}
	<div class="card">
		<table>
			<thead>
				<tr>
					<th>ID</th>
					<th>Event Type</th>
					<th>Context</th>
					<th>Interval</th>
				</tr>
			</thead>
			<tbody>
				{#each timers as timer}
					<tr>
						<td class="id">{timer.id.slice(0, 8)}...</td>
						<td>{timer.event_type}</td>
						<td>{timer.context || '-'}</td>
						<td>{formatInterval(timer.interval_secs)}</td>
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

	.error {
		color: var(--error);
	}

	.muted {
		color: var(--text-muted);
	}

	.id {
		font-family: monospace;
		color: var(--text-muted);
	}
</style>
