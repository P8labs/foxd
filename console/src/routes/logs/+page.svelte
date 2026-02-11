<script lang="ts">
  import { onMount } from "svelte";
  import { api, type LogEntry } from "$lib/api";
  import Loading from "$lib/components/Loading.svelte";
  import Alert from "$lib/components/Alert.svelte";

  let logs: LogEntry[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let filterLevel = $state<"all" | "info" | "warning" | "error" | "debug">(
    "all",
  );
  let searchQuery = $state("");

  async function loadLogs() {
    try {
      loading = true;
      error = "";
      const data = await api.getLogs();
      logs = data.logs;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load logs";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadLogs();
    const interval = setInterval(loadLogs, 10000);
    return () => clearInterval(interval);
  });

  const filteredLogs = $derived(() => {
    let filtered = logs;

    if (filterLevel !== "all") {
      filtered = filtered.filter((log) => log.level === filterLevel);
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (log) =>
          log.message.toLowerCase().includes(query) ||
          log.category.toLowerCase().includes(query) ||
          log.details?.toLowerCase().includes(query),
      );
    }

    return filtered;
  });

  function getLevelColor(level: LogEntry["level"]): string {
    switch (level) {
      case "info":
        return "info";
      case "warning":
        return "warning";
      case "error":
        return "danger";
      case "debug":
        return "secondary";
      default:
        return "secondary";
    }
  }

  function formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }
</script>

<svelte:head>
  <title>Logs - Fox Daemon Console</title>
</svelte:head>

<div class="page-header">
  <div>
    <h1 class="page-title">Logs</h1>
    <p class="page-subtitle">Monitor system events and activities</p>
  </div>
  <button class="btn btn-primary" onclick={loadLogs}> Refresh </button>
</div>

<div class="card filter-bar">
  <input
    type="text"
    placeholder="Search logs..."
    class="form-input"
    bind:value={searchQuery}
    style="flex: 1; min-width: 200px;"
  />

  <div class="filter-buttons">
    <button
      class="btn {filterLevel === 'all' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterLevel = "all")}
    >
      All ({logs.length})
    </button>
    <button
      class="btn {filterLevel === 'info' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterLevel = "info")}
    >
      Info ({logs.filter((l) => l.level === "info").length})
    </button>
    <button
      class="btn {filterLevel === 'warning' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterLevel = "warning")}
    >
      Warning ({logs.filter((l) => l.level === "warning").length})
    </button>
    <button
      class="btn {filterLevel === 'error' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterLevel = "error")}
    >
      Error ({logs.filter((l) => l.level === "error").length})
    </button>
    <button
      class="btn {filterLevel === 'debug' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterLevel = "debug")}
    >
      Debug ({logs.filter((l) => l.level === "debug").length})
    </button>
  </div>
</div>

{#if loading}
  <Loading message="Loading logs..." />
{:else if error}
  <Alert type="error" message={error} />
  <button class="btn btn-primary" onclick={loadLogs}>Retry</button>
{:else}
  <div class="card">
    {#if filteredLogs().length > 0}
      <div class="table-container">
        <table class="table">
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>Level</th>
              <th>Category</th>
              <th>Message</th>
              <th>Details</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredLogs() as log}
              <tr>
                <td
                  class="text-muted"
                  style="font-size: 0.8125rem; white-space: nowrap;"
                  >{formatTimestamp(log.timestamp)}</td
                >
                <td>
                  <span
                    class="badge badge-{getLevelColor(log.level)}"
                    style="text-transform: uppercase;"
                  >
                    {log.level}
                  </span>
                </td>
                <td style="font-weight: 500;">{log.category}</td>
                <td>{log.message}</td>
                <td class="text-muted" style="font-size: 0.875rem;">
                  {log.details || "-"}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-state-title">No logs found</div>
        <p class="empty-state-text">
          {#if searchQuery || filterLevel !== "all"}
            No logs match your current filters
          {:else}
            No logs have been recorded yet
          {/if}
        </p>
        {#if searchQuery || filterLevel !== "all"}
          <button
            class="btn btn-secondary"
            onclick={() => {
              searchQuery = "";
              filterLevel = "all";
            }}
          >
            Clear Filters
          </button>
        {/if}
      </div>
    {/if}
  </div>
{/if}
