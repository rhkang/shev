<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Schedule } from '$lib/api';

	let schedules = $state<Schedule[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Modal state
	let showModal = $state(false);
	let editingSchedule = $state<Schedule | null>(null);
	let formData = $state({
		event_type: '',
		scheduled_time: '',
		context: '',
		periodic: false
	});

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

	function toDatetimeLocal(isoString: string): string {
		const date = new Date(isoString);
		// Get local time in YYYY-MM-DDTHH:MM format
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		return `${year}-${month}-${day}T${hours}:${minutes}`;
	}

	function getDefaultTime(): string {
		const date = new Date();
		date.setMinutes(date.getMinutes() + 30); // Default to 30 mins from now
		return toDatetimeLocal(date.toISOString());
	}

	function openCreateModal() {
		editingSchedule = null;
		formData = {
			event_type: '',
			scheduled_time: getDefaultTime(),
			context: '',
			periodic: false
		};
		showModal = true;
	}

	function openEditModal(schedule: Schedule) {
		editingSchedule = schedule;
		formData = {
			event_type: schedule.event_type,
			scheduled_time: toDatetimeLocal(schedule.scheduled_time),
			context: schedule.context,
			periodic: schedule.periodic
		};
		showModal = true;
	}

	function closeModal() {
		showModal = false;
		editingSchedule = null;
	}

	async function saveSchedule() {
		try {
			// Convert local datetime to ISO string
			const scheduledTime = new Date(formData.scheduled_time).toISOString();

			if (editingSchedule) {
				await api.updateSchedule(editingSchedule.event_type, {
					scheduled_time: scheduledTime,
					context: formData.context,
					periodic: formData.periodic
				});
			} else {
				await api.createSchedule({
					event_type: formData.event_type,
					scheduled_time: scheduledTime,
					context: formData.context,
					periodic: formData.periodic
				});
			}
			closeModal();
			await fetchSchedules();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to save schedule');
		}
	}

	async function deleteSchedule(event_type: string) {
		if (!confirm(`Delete schedule "${event_type}"?`)) return;

		try {
			await api.deleteSchedule(event_type);
			await fetchSchedules();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to delete schedule');
		}
	}

	onMount(fetchSchedules);
</script>

<div class="header">
	<h2>Schedules</h2>
	<div class="controls">
		<button onclick={openCreateModal}>Add Schedule</button>
		<button class="secondary" onclick={fetchSchedules}>Refresh</button>
	</div>
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
					<th>Actions</th>
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
						<td>
							<div class="actions">
								<button class="small secondary" onclick={() => openEditModal(schedule)}>Edit</button>
								<button class="small danger" onclick={() => deleteSchedule(schedule.event_type)}>Delete</button>
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
			<h3>{editingSchedule ? 'Edit Schedule' : 'Add Schedule'}</h3>
			<form onsubmit={(e) => { e.preventDefault(); saveSchedule(); }}>
				<div class="form-group">
					<label for="event_type">Event Type</label>
					<input
						id="event_type"
						type="text"
						bind:value={formData.event_type}
						disabled={!!editingSchedule}
						required
						placeholder="e.g., daily.report"
					/>
				</div>
				<div class="form-group">
					<label for="scheduled_time">Scheduled Time</label>
					<input
						id="scheduled_time"
						type="datetime-local"
						bind:value={formData.scheduled_time}
						required
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
				<div class="form-group checkbox-group">
					<label>
						<input
							type="checkbox"
							bind:checked={formData.periodic}
						/>
						Repeat daily at this time
					</label>
				</div>
				<div class="modal-actions">
					<button type="button" class="secondary" onclick={closeModal}>Cancel</button>
					<button type="submit">{editingSchedule ? 'Update' : 'Create'}</button>
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

	.badge {
		background: var(--bg-tertiary);
	}

	.badge.periodic {
		background: var(--primary);
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

	.checkbox-group label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
	}

	.checkbox-group input {
		width: auto;
		margin: 0;
	}

	.modal-actions {
		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
		margin-top: 1.5rem;
	}
</style>
