<script lang="ts">
  import { onMount } from "svelte";
  import { api, getStatusColor, type Device } from "$lib/api";
  import Loading from "$lib/components/Loading.svelte";
  import Alert from "$lib/components/Alert.svelte";

  let devices: Device[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let filterStatus = $state<"all" | "online" | "offline">("all");
  let searchQuery = $state("");

  async function loadDevices() {
    try {
      loading = true;
      error = "";
      const data = await api.getDevices();
      devices = data.devices;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load devices";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDevices();
    const interval = setInterval(loadDevices, 15000);
    return () => clearInterval(interval);
  });

  const filteredDevices = $derived(() => {
    let filtered = devices;

    if (filterStatus !== "all") {
      filtered = filtered.filter((d) => d.status === filterStatus);
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (d) =>
          d.mac_address.toLowerCase().includes(query) ||
          d.ip_address?.toLowerCase().includes(query) ||
          d.hostname?.toLowerCase().includes(query),
      );
    }

    return filtered;
  });
</script>

<svelte:head>
  <title>Devices - Fox Daemon Console</title>
</svelte:head>

<div class="page-header">
  <div>
    <h1 class="page-title">Devices</h1>
    <p class="page-subtitle">Monitor all devices on your network</p>
  </div>
  <button class="btn btn-primary" onclick={loadDevices}> Refresh </button>
</div>

<div class="card filter-bar">
  <input
    type="text"
    placeholder="Search by MAC, IP, or hostname..."
    class="form-input"
    bind:value={searchQuery}
    style="flex: 1; min-width: 200px;"
  />

  <div class="filter-buttons">
    <button
      class="btn {filterStatus === 'all' ? 'btn-primary' : 'btn-secondary'}"
      onclick={() => (filterStatus = "all")}
    >
      All ({devices.length})
    </button>
    <button
      class="btn {filterStatus === 'online' ? 'btn-success' : 'btn-secondary'}"
      onclick={() => (filterStatus = "online")}
    >
      Online ({devices.filter((d) => d.status === "online").length})
    </button>
    <button
      class="btn {filterStatus === 'offline' ? 'btn-danger' : 'btn-secondary'}"
      onclick={() => (filterStatus = "offline")}
    >
      Offline ({devices.filter((d) => d.status === "offline").length})
    </button>
  </div>
</div>

{#if loading}
  <Loading message="Loading devices..." />
{:else if error}
  <Alert type="error" message={error} />
  <button class="btn btn-primary" onclick={loadDevices}>Retry</button>
{:else}
  <div class="card">
    {#if filteredDevices().length > 0}
      <div class="table-container">
        <table class="table">
          <thead>
            <tr>
              <th>Status</th>
              <th>MAC Address</th>
              <th>IP Address</th>
              <th>Hostname</th>
              <th>First Seen</th>
              <th>Last Seen</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredDevices() as device}
              <tr>
                <td>
                  <span
                    class="badge badge-{device.status === 'online'
                      ? 'success'
                      : 'danger'}"
                  >
                    {device.status}
                  </span>
                </td>
                <td class="text-mono">{device.mac_address}</td>
                <td class="text-mono">{device.ip_address || "-"}</td>
                <td>{device.hostname || "-"}</td>
                <td class="text-muted" style="font-size: 0.8125rem"
                  >{new Date(device.first_seen).toLocaleString()}</td
                >
                <td class="text-muted" style="font-size: 0.8125rem"
                  >{new Date(device.last_seen).toLocaleString()}</td
                >
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-state-title">No devices found</div>
        <p class="empty-state-text">
          {#if searchQuery || filterStatus !== "all"}
            No devices match your current filters
          {:else}
            No devices have been discovered yet
          {/if}
        </p>
        {#if searchQuery || filterStatus !== "all"}
          <button
            class="btn btn-secondary"
            onclick={() => {
              searchQuery = "";
              filterStatus = "all";
            }}
          >
            Clear Filters
          </button>
        {/if}
      </div>
    {/if}
  </div>
{/if}
