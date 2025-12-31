<script lang="ts">
	import { api } from '$lib/api';

	let eventType = $state('');
	let context = $state('');
	let loading = $state(false);
	let result = $state<{ success: boolean; message: string } | null>(null);

	async function triggerEvent() {
		if (!eventType.trim()) {
			result = { success: false, message: 'Event type is required' };
			return;
		}

		loading = true;
		result = null;

		try {
			const response = await api.createEvent(eventType.trim(), context.trim());
			result = { success: true, message: `Event queued: ${response.id}` };
			eventType = '';
			context = '';
		} catch (e) {
			result = { success: false, message: e instanceof Error ? e.message : 'Failed to trigger event' };
		} finally {
			loading = false;
		}
	}
</script>

<h2>Trigger Event</h2>

<div class="card form-card">
	<form onsubmit={(e) => { e.preventDefault(); triggerEvent(); }}>
		<div class="field">
			<label for="event-type">Event Type</label>
			<input
				id="event-type"
				type="text"
				bind:value={eventType}
				placeholder="e.g., backup, deploy, cleanup"
				disabled={loading}
			/>
		</div>

		<div class="field">
			<label for="context">Context (optional)</label>
			<textarea
				id="context"
				bind:value={context}
				placeholder="Additional context for the event handler"
				rows="4"
				disabled={loading}
			></textarea>
		</div>

		<button type="submit" disabled={loading}>
			{loading ? 'Triggering...' : 'Trigger Event'}
		</button>
	</form>

	{#if result}
		<div class="result" class:success={result.success} class:error={!result.success}>
			{result.message}
		</div>
	{/if}
</div>

<style>
	h2 {
		margin-bottom: 1.5rem;
	}

	.form-card {
		max-width: 500px;
	}

	.field {
		margin-bottom: 1rem;
	}

	label {
		display: block;
		margin-bottom: 0.5rem;
		color: var(--text-muted);
		font-size: 0.875rem;
	}

	button {
		width: 100%;
	}

	.result {
		margin-top: 1rem;
		padding: 0.75rem;
		border-radius: 0.375rem;
	}

	.result.success {
		background: rgba(34, 197, 94, 0.1);
		color: var(--success);
		border: 1px solid var(--success);
	}

	.result.error {
		background: rgba(239, 68, 68, 0.1);
		color: var(--error);
		border: 1px solid var(--error);
	}
</style>
