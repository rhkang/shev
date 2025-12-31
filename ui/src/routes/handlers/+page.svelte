<script lang="ts">
    import { onMount } from "svelte";
    import { api, type EventHandler } from "$lib/api";

    let handlers = $state<EventHandler[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    // Modal state
    let showModal = $state(false);
    let editingHandler = $state<EventHandler | null>(null);
    let formData = $state({
        event_type: "",
        shell: "pwsh",
        command: "",
        timeout: "" as string | number,
        env: "", // KEY=VALUE pairs, one per line
    });

    const SHELLS = ["pwsh", "bash", "sh"];

    async function fetchHandlers() {
        try {
            handlers = await api.getHandlers();
            error = null;
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to fetch handlers";
        } finally {
            loading = false;
        }
    }

    function envToString(env: Record<string, string>): string {
        return Object.entries(env)
            .map(([k, v]) => `${k}=${v}`)
            .join("\n");
    }

    function stringToEnv(str: string): Record<string, string> {
        const env: Record<string, string> = {};
        for (const line of str.split("\n")) {
            const trimmed = line.trim();
            if (!trimmed) continue;
            const idx = trimmed.indexOf("=");
            if (idx > 0) {
                env[trimmed.substring(0, idx)] = trimmed.substring(idx + 1);
            }
        }
        return env;
    }

    function openCreateModal() {
        editingHandler = null;
        formData = {
            event_type: "",
            shell: "pwsh",
            command: "",
            timeout: "",
            env: "",
        };
        showModal = true;
    }

    function openEditModal(handler: EventHandler) {
        editingHandler = handler;
        formData = {
            event_type: handler.event_type,
            shell: handler.shell,
            command: handler.command,
            timeout: handler.timeout ?? "",
            env: envToString(handler.env),
        };
        showModal = true;
    }

    function closeModal() {
        showModal = false;
        editingHandler = null;
    }

    async function saveHandler() {
        try {
            const timeout =
                formData.timeout === "" ? undefined : Number(formData.timeout);
            const env = stringToEnv(formData.env);

            if (editingHandler) {
                await api.updateHandler(editingHandler.event_type, {
                    shell: formData.shell,
                    command: formData.command,
                    timeout: timeout ?? null,
                    env,
                });
            } else {
                await api.createHandler({
                    event_type: formData.event_type,
                    shell: formData.shell,
                    command: formData.command,
                    timeout,
                    env,
                });
            }
            closeModal();
            await fetchHandlers();
        } catch (e) {
            alert(e instanceof Error ? e.message : "Failed to save handler");
        }
    }

    async function deleteHandler(event_type: string) {
        if (
            !confirm(
                `Delete handler "${event_type}"? This may leave orphaned timers/schedules.`,
            )
        )
            return;

        try {
            await api.deleteHandler(event_type);
            await fetchHandlers();
        } catch (e) {
            alert(e instanceof Error ? e.message : "Failed to delete handler");
        }
    }

    onMount(fetchHandlers);
</script>

<div class="header">
    <h2>Event Handlers</h2>
    <div class="controls">
        <button onclick={openCreateModal}>Add Handler</button>
        <button class="secondary" onclick={fetchHandlers}>Refresh</button>
    </div>
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
                {#if Object.keys(handler.env).length > 0}
                    <div class="env-section">
                        <div class="env-label">Environment:</div>
                        {#each Object.entries(handler.env) as [key, value]}
                            <div class="env-item">{key}={value}</div>
                        {/each}
                    </div>
                {/if}
                <div class="actions">
                    <button class="small" onclick={() => openEditModal(handler)}
                        >Edit</button
                    >
                    <button
                        class="small danger"
                        onclick={() => deleteHandler(handler.event_type)}
                        >Delete</button
                    >
                </div>
            </div>
        {/each}
    </div>
{/if}

{#if showModal}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="modal-backdrop"
        role="button"
        tabindex="-1"
        onclick={closeModal}
        onkeydown={(e) => e.key === "Escape" && closeModal()}
    >
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="modal"
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            <h3>{editingHandler ? "Edit Handler" : "Add Handler"}</h3>
            <form
                onsubmit={(e) => {
                    e.preventDefault();
                    saveHandler();
                }}
            >
                <div class="form-group">
                    <label for="event_type">Event Type</label>
                    <input
                        id="event_type"
                        type="text"
                        bind:value={formData.event_type}
                        disabled={!!editingHandler}
                        required
                        placeholder="e.g., claude-daily-wrapup"
                    />
                </div>
                <div class="form-group">
                    <label for="shell">Shell</label>
                    <select id="shell" bind:value={formData.shell}>
                        {#each SHELLS as shell}
                            <option value={shell}>{shell}</option>
                        {/each}
                    </select>
                </div>
                <div class="form-group">
                    <label for="command">Command</label>
                    <textarea
                        id="command"
                        bind:value={formData.command}
                        required
                        rows="4"
                        placeholder="echo $EVENT_CONTEXT"
                    ></textarea>
                </div>
                <div class="form-group">
                    <label for="timeout">Timeout (seconds, optional)</label>
                    <input
                        id="timeout"
                        type="number"
                        bind:value={formData.timeout}
                        min="1"
                        placeholder="Leave empty for no timeout"
                    />
                </div>
                <div class="form-group">
                    <label for="env"
                        >Environment Variables (one per line: KEY=VALUE)</label
                    >
                    <textarea
                        id="env"
                        bind:value={formData.env}
                        rows="3"
                        placeholder={"MY_VAR=value\nANOTHER=value2"}
                    ></textarea>
                </div>
                <div class="modal-actions">
                    <button type="button" class="secondary" onclick={closeModal}
                        >Cancel</button
                    >
                    <button type="submit"
                        >{editingHandler ? "Update" : "Create"}</button
                    >
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
        margin-bottom: 0.75rem;
    }

    .env-section {
        font-size: 0.875rem;
        margin-bottom: 0.75rem;
        padding: 0.5rem;
        background: var(--bg-tertiary);
        border-radius: 4px;
    }

    .env-label {
        color: var(--text-muted);
        margin-bottom: 0.25rem;
    }

    .env-item {
        font-family: monospace;
        font-size: 0.8rem;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
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
        max-width: 500px;
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

    .form-group textarea {
        font-family: monospace;
    }

    .modal-actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
        margin-top: 1.5rem;
    }
</style>
