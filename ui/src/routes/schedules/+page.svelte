<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Schedule } from '$lib/api';

	let schedules = $state<Schedule[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function fetchSchedules() {
		try {
			schedules = await api.getSchedules();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch schedules';
		} finally {
			loading = false;
		}
	}

	function formatTime(time: string): string {
		return new Date(time).toLocaleString();
	}

	onMount(fetchSchedules);
</script>

<div class="header">
	<h2>Schedules</h2>
	<button class="secondary" onclick={fetchSchedules}>Refresh</button>
</div>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if schedules.length === 0}
	<p class="muted">No schedules configured</p>
{:else}
	<div class="card">
		<table>
			<thead>
				<tr>
					<th>ID</th>
					<th>Event Type</th>
					<th>Context</th>
					<th>Scheduled Time</th>
					<th>Type</th>
				</tr>
			</thead>
			<tbody>
				{#each schedules as schedule}
					<tr>
						<td class="id">{schedule.id.slice(0, 8)}...</td>
						<td>{schedule.event_type}</td>
						<td>{schedule.context || '-'}</td>
						<td>{formatTime(schedule.scheduled_time)}</td>
						<td>
							<span class="badge" class:periodic={schedule.periodic}>
								{schedule.periodic ? 'Daily' : 'One-time'}
							</span>
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

	.badge {
		background: var(--bg-tertiary);
	}

	.badge.periodic {
		background: var(--primary);
	}
</style>
