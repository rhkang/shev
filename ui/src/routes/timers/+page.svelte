<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Timer } from '$lib/api';

	let timers = $state<Timer[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Modal state
	let showModal = $state(false);
	let editingTimer = $state<Timer | null>(null);
	let formData = $state({
		event_type: '',
		interval_secs: 60,
		context: ''
	});

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

	function openCreateModal() {
		editingTimer = null;
		formData = { event_type: '', interval_secs: 60, context: '' };
		showModal = true;
	}

	function openEditModal(timer: Timer) {
		editingTimer = timer;
		formData = {
			event_type: timer.event_type,
			interval_secs: timer.interval_secs,
			context: timer.context
		};
		showModal = true;
	}

	function closeModal() {
		showModal = false;
		editingTimer = null;
	}

	async function saveTimer() {
		try {
			if (editingTimer) {
				await api.updateTimer(editingTimer.event_type, {
					interval_secs: formData.interval_secs,
					context: formData.context
				});
			} else {
				await api.createTimer({
					event_type: formData.event_type,
					interval_secs: formData.interval_secs,
					context: formData.context
				});
			}
			closeModal();
			await fetchTimers();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to save timer');
		}
	}

	async function deleteTimer(event_type: string) {
		if (!confirm(`Delete timer "${event_type}"?`)) return;

		try {
			await api.deleteTimer(event_type);
			await fetchTimers();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to delete timer');
		}
	}

	onMount(fetchTimers);
</script>

<div class="header">
	<h2>Timers</h2>
	<div class="controls">
		<button onclick={openCreateModal}>Add Timer</button>
		<button class="secondary" onclick={fetchTimers}>Refresh</button>
	</div>
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
					<th>Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each timers as timer}
					<tr>
						<td class="id">{timer.id.slice(0, 8)}...</td>
						<td>{timer.event_type}</td>
						<td>{timer.context || '-'}</td>
						<td>{formatInterval(timer.interval_secs)}</td>
						<td>
							<div class="actions">
								<button class="small secondary" onclick={() => openEditModal(timer)}>Edit</button>
								<button class="small danger" onclick={() => deleteTimer(timer.event_type)}>Delete</button>
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

{#if showModal}
	<div class="modal-backdrop" onclick={closeModal}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<h3>{editingTimer ? 'Edit Timer' : 'Add Timer'}</h3>
			<form onsubmit={(e) => { e.preventDefault(); saveTimer(); }}>
				<div class="form-group">
					<label for="event_type">Event Type</label>
					<input
						id="event_type"
						type="text"
						bind:value={formData.event_type}
						disabled={!!editingTimer}
						required
						placeholder="e.g., heartbeat"
					/>
				</div>
				<div class="form-group">
					<label for="interval_secs">Interval (seconds)</label>
					<input
						id="interval_secs"
						type="number"
						bind:value={formData.interval_secs}
						required
						min="1"
					/>
				</div>
				<div class="form-group">
					<label for="context">Context (optional)</label>
					<input
						id="context"
						type="text"
						bind:value={formData.context}
						placeholder="Additional context data"
					/>
				</div>
				<div class="modal-actions">
					<button type="button" class="secondary" onclick={closeModal}>Cancel</button>
					<button type="submit">{editingTimer ? 'Update' : 'Create'}</button>
				</div>
			</form>
		</div>
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

	.actions {
		display: flex;
		gap: 0.5rem;
	}

	button.small {
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
	}

	/* Modal styles */
	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--bg-secondary);
		border-radius: 8px;
		padding: 1.5rem;
		width: 90%;
		max-width: 400px;
		max-height: 90vh;
		overflow-y: auto;
	}

	.modal h3 {
		margin-bottom: 1rem;
	}

	.form-group {
		margin-bottom: 1rem;
	}

	.form-group label {
		display: block;
		margin-bottom: 0.25rem;
		font-size: 0.875rem;
		color: var(--text-muted);
	}

	.modal-actions {
		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
		margin-top: 1.5rem;
	}
</style>
