<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Shev - Event-Driven Shell Executor</title>
</svelte:head>

<div class="layout">
	<nav class="sidebar">
		<div class="logo">
			<h1>Shev</h1>
		</div>
		<ul class="nav-links">
			<li><a href="/">Dashboard</a></li>
			<li><a href="/jobs">Jobs</a></li>
			<li><a href="/handlers">Handlers</a></li>
			<li><a href="/timers">Timers</a></li>
			<li><a href="/schedules">Schedules</a></li>
			<li><a href="/events">Trigger Event</a></li>
		</ul>
	</nav>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	.layout {
		display: flex;
		min-height: 100vh;
	}

	.sidebar {
		width: 220px;
		background: var(--bg-secondary);
		border-right: 1px solid var(--border);
		padding: 1.5rem;
		flex-shrink: 0;
	}

	.logo h1 {
		font-size: 1.5rem;
		margin-bottom: 2rem;
		color: var(--primary);
	}

	.nav-links {
		list-style: none;
	}

	.nav-links li {
		margin-bottom: 0.5rem;
	}

	.nav-links a {
		display: block;
		padding: 0.5rem 0.75rem;
		border-radius: 0.375rem;
		color: var(--text-muted);
		transition: all 0.2s;
	}

	.nav-links a:hover {
		background: var(--bg-tertiary);
		color: var(--text);
		text-decoration: none;
	}

	.content {
		flex: 1;
		padding: 2rem;
		overflow-y: auto;
	}
</style>
