<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Config, type NotificationChannel } from "$lib/api";
  import Loading from "$lib/components/Loading.svelte";
  import Alert from "$lib/components/Alert.svelte";
  import Modal from "$lib/components/Modal.svelte";

  let config: Config | null = $state(null);
  let loading = $state(true);
  let error = $state("");
  let success = $state("");
  let showChannelModal = $state(false);

  let channelForm = $state<{
    type: "telegram" | "ntfy" | "webhook";
    bot_token?: string;
    chat_id?: string;
    server_url?: string;
    topic?: string;
    token?: string;
    url?: string;
  }>({
    type: "telegram",
  });

  async function loadConfig() {
    try {
      loading = true;
      error = "";
      config = await api.getConfig();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to load configuration";
    } finally {
      loading = false;
    }
  }

  function openAddChannelModal() {
    channelForm = { type: "telegram" };
    showChannelModal = true;
  }

  async function handleAddChannel(e: Event) {
    e.preventDefault();
    if (!config) return;

    try {
      error = "";

      let channel: NotificationChannel;

      if (channelForm.type === "telegram") {
        if (!channelForm.bot_token || !channelForm.chat_id) {
          error = "Bot token and chat ID are required";
          return;
        }
        channel = {
          type: "telegram",
          bot_token: channelForm.bot_token,
          chat_id: channelForm.chat_id,
        };
      } else if (channelForm.type === "ntfy") {
        if (!channelForm.server_url || !channelForm.topic) {
          error = "Server URL and topic are required";
          return;
        }
        channel = {
          type: "ntfy",
          server_url: channelForm.server_url,
          topic: channelForm.topic,
          token: channelForm.token || undefined,
        };
      } else {
        if (!channelForm.url) {
          error = "Webhook URL is required";
          return;
        }
        channel = {
          type: "webhook",
          url: channelForm.url,
          headers: undefined,
        };
      }

      const updatedChannels = [...config.notifications, channel];
      await api.updateConfig({ notifications: updatedChannels });

      success = "Notification channel added successfully";
      showChannelModal = false;
      await loadConfig();
      setTimeout(() => (success = ""), 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to add channel";
    }
  }

  async function removeChannel(index: number) {
    if (!config) return;
    if (!confirm("Are you sure you want to remove this notification channel?"))
      return;

    try {
      error = "";
      const updatedChannels = config.notifications.filter(
        (_, i) => i !== index,
      );
      await api.updateConfig({ notifications: updatedChannels });

      success = "Notification channel removed successfully";
      await loadConfig();
      setTimeout(() => (success = ""), 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to remove channel";
    }
  }

  onMount(() => {
    loadConfig();
  });
</script>

<svelte:head>
  <title>Configuration - Fox Daemon Console</title>
</svelte:head>

<!-- Header -->
<div class="page-header">
  <div>
    <h1 class="page-title">Configuration</h1>
    <p class="page-subtitle">
      Manage daemon settings and notification channels
    </p>
  </div>
  <button class="btn btn-primary" onclick={loadConfig}> Refresh </button>
</div>

<!-- Alerts -->
{#if success}
  <Alert type="success" message={success} />
{/if}
{#if error && !showChannelModal}
  <Alert type="error" message={error} />
{/if}

{#if loading}
  <Loading message="Loading configuration..." />
{:else if config}
  <!-- Daemon Settings -->
  <div class="card">
    <h3 class="card-title">Daemon Settings</h3>
    <div style="margin-top: 1rem;">
      <div class="flex justify-between items-center mb-2">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Network Interface</span
        >
        <span class="text-mono" style="font-weight: 600;"
          >{config.daemon.interface}</span
        >
      </div>
      <div class="flex justify-between items-center mb-2">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Neighbor Check Interval</span
        >
        <span style="font-weight: 600;"
          >{config.daemon.neighbor_check_interval_secs}s</span
        >
      </div>
      <div class="flex justify-between items-center mb-2">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Device Timeout</span
        >
        <span style="font-weight: 600;"
          >{config.daemon.device_timeout_secs}s</span
        >
      </div>
      {#if config.daemon.capture_filter}
        <div class="flex justify-between items-center">
          <span style="font-size: 0.875rem; color: var(--text-secondary);"
            >Capture Filter</span
          >
          <span class="text-mono" style="font-size: 0.75rem;"
            >{config.daemon.capture_filter}</span
          >
        </div>
      {/if}
    </div>
    <p style="margin-top: 1rem; font-size: 0.75rem; color: var(--text-muted);">
      Note: Daemon settings require a restart to take effect. Edit config.toml
      on the server.
    </p>
  </div>

  <!-- API Settings -->
  <div class="card">
    <h3 class="card-title">API Settings</h3>
    <div style="margin-top: 1rem;">
      <div class="flex justify-between items-center mb-2">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Host</span
        >
        <span class="text-mono" style="font-weight: 600;"
          >{config.api.host}</span
        >
      </div>
      <div class="flex justify-between items-center">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Port</span
        >
        <span style="font-weight: 600;">{config.api.port}</span>
      </div>
    </div>
  </div>

  <div class="card">
    <h3 class="card-title">Database</h3>
    <div style="margin-top: 1rem;">
      <div class="flex justify-between items-center">
        <span style="font-size: 0.875rem; color: var(--text-secondary);"
          >Database Path</span
        >
        <span class="text-mono" style="font-size: 0.8125rem;"
          >{config.database.path}</span
        >
      </div>
    </div>
  </div>

  <div class="card">
    <div class="card-header">
      <h3 class="card-title">Notification Channels</h3>
      <button class="btn btn-primary btn-sm" onclick={openAddChannelModal}>
        Add Channel
      </button>
    </div>

    {#if config.notifications.length > 0}
      <div class="table-container">
        <table class="table">
          <thead>
            <tr>
              <th>Type</th>
              <th>Configuration</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each config.notifications as channel, index}
              <tr>
                <td>
                  <span
                    class="badge badge-info"
                    style="text-transform: capitalize;"
                  >
                    {channel.type}
                  </span>
                </td>
                <td>
                  {#if channel.type === "telegram"}
                    <div style="font-size: 0.8125rem;">
                      <div>
                        Chat ID: <span class="text-mono">{channel.chat_id}</span
                        >
                      </div>
                      <div class="text-muted">
                        Token: {channel.bot_token?.substring(0, 20)}...
                      </div>
                    </div>
                  {:else if channel.type === "ntfy"}
                    <div style="font-size: 0.8125rem;">
                      <div>
                        Topic: <span class="text-mono">{channel.topic}</span>
                      </div>
                      <div class="text-muted">{channel.server_url}</div>
                    </div>
                  {:else if channel.type === "webhook"}
                    <div style="font-size: 0.8125rem;">
                      <span class="text-mono">{channel.url}</span>
                    </div>
                  {/if}
                </td>
                <td>
                  <button
                    class="btn btn-danger btn-sm"
                    onclick={() => removeChannel(index)}
                  >
                    Remove
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-state-title">No notification channels configured</div>
        <p class="empty-state-text">
          Add a notification channel to receive alerts
        </p>
        <button class="btn btn-primary" onclick={openAddChannelModal}
          >Add First Channel</button
        >
      </div>
    {/if}
  </div>
{/if}

<Modal
  bind:open={showChannelModal}
  onClose={() => (showChannelModal = false)}
  title="Add Notification Channel"
>
  <form onsubmit={handleAddChannel}>
    <div class="form-group">
      <label class="form-label" for="channel_type">Channel Type</label>
      <select
        id="channel_type"
        class="form-select"
        bind:value={channelForm.type}
        required
      >
        <option value="telegram">Telegram Bot</option>
        <option value="ntfy">ntfy.sh</option>
        <option value="webhook">Webhook</option>
      </select>
    </div>

    {#if channelForm.type === "telegram"}
      <div class="form-group">
        <label class="form-label" for="bot_token">Bot Token</label>
        <input
          type="text"
          id="bot_token"
          class="form-input"
          bind:value={channelForm.bot_token}
          required
          placeholder="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
        />
      </div>
      <div class="form-group">
        <label class="form-label" for="chat_id">Chat ID</label>
        <input
          type="text"
          id="chat_id"
          class="form-input"
          bind:value={channelForm.chat_id}
          required
          placeholder="123456789"
        />
      </div>
    {:else if channelForm.type === "ntfy"}
      <div class="form-group">
        <label class="form-label" for="server_url">Server URL</label>
        <input
          type="url"
          id="server_url"
          class="form-input"
          bind:value={channelForm.server_url}
          required
          placeholder="https://ntfy.sh"
        />
      </div>
      <div class="form-group">
        <label class="form-label" for="topic">Topic</label>
        <input
          type="text"
          id="topic"
          class="form-input"
          bind:value={channelForm.topic}
          required
          placeholder="foxd-alerts"
        />
      </div>
      <div class="form-group">
        <label class="form-label" for="token">Token (optional)</label>
        <input
          type="text"
          id="token"
          class="form-input"
          bind:value={channelForm.token}
          placeholder="tk_xxxxxxxxxxxx"
        />
      </div>
    {:else if channelForm.type === "webhook"}
      <div class="form-group">
        <label class="form-label" for="url">Webhook URL</label>
        <input
          type="url"
          id="url"
          class="form-input"
          bind:value={channelForm.url}
          required
          placeholder="https://example.com/webhook"
        />
      </div>
    {/if}

    {#if error}
      <div style="margin-top: 1rem;">
        <Alert type="error" message={error} />
      </div>
    {/if}

    <div class="modal-footer">
      <button
        type="button"
        class="btn btn-secondary"
        onclick={() => (showChannelModal = false)}
      >
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">Add Channel</button>
    </div>
  </form>
</Modal>
