<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type EventHandler } from '$lib/api';

	let handlers = $state<EventHandler[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function fetchHandlers() {
		try {
			handlers = await api.getHandlers();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch handlers';
		} finally {
			loading = false;
		}
	}

	onMount(fetchHandlers);
</script>

<div class="header">
	<h2>Event Handlers</h2>
	<button class="secondary" onclick={fetchHandlers}>Refresh</button>
</div>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="error">{error}</p>
{:else if handlers.length === 0}
	<p class="muted">No handlers configured</p>
{:else}
	<div class="handlers-grid">
		{#each handlers as handler}
			<div class="card">
				<div class="handler-header">
					<span class="event-type">{handler.event_type}</span>
					<span class="badge shell">{handler.shell}</span>
				</div>
				<div class="handler-id">ID: {handler.id}</div>
				{#if handler.timeout}
					<div class="timeout">Timeout: {handler.timeout}s</div>
				{/if}
				<pre class="command">{handler.command}</pre>
			</div>
		{/each}
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

	.handlers-grid {
		display: grid;
		gap: 1rem;
	}

	.handler-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.event-type {
		font-weight: 600;
		font-size: 1.1rem;
	}

	.badge.shell {
		background: var(--bg-tertiary);
	}

	.handler-id {
		font-size: 0.75rem;
		color: var(--text-muted);
		margin-bottom: 0.5rem;
	}

	.timeout {
		font-size: 0.875rem;
		color: var(--text-muted);
		margin-bottom: 0.5rem;
	}

	.command {
		margin-top: 0.5rem;
	}
</style>
