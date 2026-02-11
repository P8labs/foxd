<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    formatUptime,
    type Metrics,
    type Device,
    type HealthResponse,
  } from "$lib/api";
  import StatCard from "$lib/components/StatCard.svelte";
  import Loading from "$lib/components/Loading.svelte";
  import Alert from "$lib/components/Alert.svelte";

  let metrics: Metrics | null = $state(null);
  let recentDevices: Device[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let success = $state("");
  let healthStatus = $state<HealthResponse | null>(null);

  async function loadData() {
    try {
      loading = true;
      error = "";

      const [metricsData, devicesData, health] = await Promise.all([
        api.getMetrics(),
        api.getDevices(),
        api.getHealth(),
      ]);

      metrics = metricsData;
      recentDevices = devicesData.devices.slice(0, 5);
      healthStatus = health;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load data";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
    const interval = setInterval(loadData, 30000);
    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>Dashboard - Fox Daemon Console</title>
</svelte:head>

<div class="page-header">
  <div>
    <h1 class="page-title">Dashboard</h1>
    <p class="page-subtitle">Overview of your network monitoring</p>
  </div>
  <div class="flex gap-2">
    {#if healthStatus}
      <div class="status-indicator">
        <span class="status-dot online"></span>
        <span>System Online</span>
      </div>
    {/if}
  </div>
</div>

{#if success}
  <Alert type="success" message={success} />
{/if}

{#if loading}
  <Loading message="Loading dashboard..." />
{:else if error}
  <Alert type="error" message={error} />
  <button class="btn btn-primary" onclick={loadData}>Retry</button>
{:else if metrics}
  <div class="stats-grid">
    <StatCard
      title="Total Devices"
      value={metrics.total_devices}
      subtitle="{metrics.online_devices} online, {metrics.offline_devices} offline"
    />
    <StatCard
      title="Online Devices"
      value={metrics.online_devices}
      subtitle="{Math.round(
        (metrics.online_devices / metrics.total_devices) * 100,
      )}% of total"
    />
    <StatCard
      title="Active Rules"
      value={metrics.enabled_rules}
      subtitle="{metrics.total_rules} total rules"
    />
    <StatCard
      title="System Uptime"
      value={formatUptime(metrics.uptime_seconds)}
      subtitle="Running smoothly"
    />
  </div>

  {#if healthStatus}
    <div class="card">
      <h3 class="card-title">System Health</h3>
      <div style="margin-top: 1rem;">
        <div class="flex justify-between items-center mb-3">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >CPU Usage</span
          >
          <div class="flex items-center gap-2" style="min-width: 8rem;">
            <div class="progress-bar" style="flex: 1;">
              <div
                class="progress-fill"
                style="width: {healthStatus.system
                  .cpu_usage_percent}%; background-color: {healthStatus.system
                  .cpu_usage_percent > 80
                  ? 'var(--danger)'
                  : 'var(--success)'};"
              ></div>
            </div>
            <span style="font-weight: 600;"
              >{healthStatus.system.cpu_usage_percent}%</span
            >
          </div>
        </div>
        <div class="flex justify-between items-center mb-3">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Memory Usage</span
          >
          <div class="flex items-center gap-2" style="min-width: 8rem;">
            <div class="progress-bar" style="flex: 1;">
              <div
                class="progress-fill"
                style="width: {healthStatus.system
                  .memory_usage_percent}%; background-color: {healthStatus
                  .system.memory_usage_percent > 80
                  ? 'var(--danger)'
                  : 'var(--success)'};"
              ></div>
            </div>
            <span style="font-weight: 600;"
              >{healthStatus.system.memory_usage_percent}%</span
            >
          </div>
        </div>
        <div class="flex justify-between items-center">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Memory</span
          >
          <span style="font-weight: 600;"
            >{healthStatus.system.used_memory_mb} MB / {healthStatus.system
              .total_memory_mb} MB</span
          >
        </div>
      </div>
    </div>
  {/if}

  <div class="stats-grid">
    <div class="card">
      <h3 class="card-title">Network Activity</h3>
      <div style="margin-top: 1rem;">
        <div class="flex justify-between items-center mb-2">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Packets Captured</span
          >
          <span style="font-weight: 600;"
            >{metrics.packets_captured.toLocaleString()}</span
          >
        </div>
        <div class="flex justify-between items-center">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Notifications Sent</span
          >
          <span style="font-weight: 600;"
            >{metrics.notifications_sent.toLocaleString()}</span
          >
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="card-title">Device Status</h3>
      <div style="margin-top: 1rem;">
        <div class="flex justify-between items-center mb-2">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Online</span
          >
          <div class="flex items-center gap-2">
            <div class="progress-bar" style="width: 6rem;">
              <div
                class="progress-fill"
                style="width: {(metrics.online_devices /
                  metrics.total_devices) *
                  100}%"
              ></div>
            </div>
            <span style="font-weight: 600;">{metrics.online_devices}</span>
          </div>
        </div>
        <div class="flex justify-between items-center">
          <span style="font-size: 0.875rem; color: var(--text); opacity: 0.6;"
            >Offline</span
          >
          <div class="flex items-center gap-2">
            <div class="progress-bar" style="width: 6rem;">
              <div
                class="progress-fill"
                style="width: {(metrics.offline_devices /
                  metrics.total_devices) *
                  100}%; background-color: var(--danger);"
              ></div>
            </div>
            <span style="font-weight: 600;">{metrics.offline_devices}</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div class="card">
    <div class="card-header">
      <h3 class="card-title">Recent Devices</h3>
      <a
        href="/devices"
        style="font-size: 0.875rem; color: var(--text); opacity: 0.6; text-decoration: none;"
        >View all →</a
      >
    </div>

    {#if recentDevices.length > 0}
      <div class="table-container">
        <table class="table">
          <thead>
            <tr>
              <th>Status</th>
              <th>MAC Address</th>
              <th>IP Address</th>
              <th>Hostname</th>
              <th>Last Seen</th>
            </tr>
          </thead>
          <tbody>
            {#each recentDevices as device}
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
                <td class="text-muted"
                  >{new Date(device.last_seen).toLocaleString()}</td
                >
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <p class="empty-state-text">No devices found</p>
      </div>
    {/if}
  </div>
{/if}
