<script lang="ts">
  import { onMount } from "svelte";
  import { api, getTriggerTypeLabel, type Rule } from "$lib/api";
  import Loading from "$lib/components/Loading.svelte";
  import Alert from "$lib/components/Alert.svelte";
  import Modal from "$lib/components/Modal.svelte";

  let rules: Rule[] = $state([]);
  let loading = $state(true);
  let error = $state("");
  let success = $state("");
  let showModal = $state(false);
  let editingRule: Rule | null = $state(null);

  let formData = $state({
    name: "",
    description: "",
    trigger_type: "new_device" as Rule["trigger_type"],
    mac_filter: "",
    enabled: true,
    notification_channels: "",
  });

  async function loadRules() {
    try {
      loading = true;
      error = "";
      const data = await api.getRules();
      rules = data.rules;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load rules";
    } finally {
      loading = false;
    }
  }

  function openCreateModal() {
    editingRule = null;
    formData = {
      name: "",
      description: "",
      trigger_type: "new_device",
      mac_filter: "",
      enabled: true,
      notification_channels: "",
    };
    showModal = true;
  }

  function openEditModal(rule: Rule) {
    editingRule = rule;
    formData = {
      name: rule.name,
      description: rule.description || "",
      trigger_type: rule.trigger_type,
      mac_filter: rule.mac_filter || "",
      enabled: rule.enabled,
      notification_channels: rule.notification_channels.join(", "),
    };
    showModal = true;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    try {
      error = "";
      success = "";

      const ruleData = {
        name: formData.name,
        description: formData.description || null,
        trigger_type: formData.trigger_type,
        mac_filter: formData.mac_filter || null,
        enabled: formData.enabled,
        notification_channels: formData.notification_channels
          .split(",")
          .map((c) => c.trim())
          .filter((c) => c),
      };

      if (editingRule) {
        await api.updateRule(editingRule.id, ruleData);
        success = "Rule updated successfully";
      } else {
        await api.createRule(ruleData);
        success = "Rule created successfully";
      }

      showModal = false;
      await loadRules();
      setTimeout(() => (success = ""), 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to save rule";
    }
  }

  async function deleteRule(id: number) {
    if (!confirm("Are you sure you want to delete this rule?")) return;

    try {
      error = "";
      await api.deleteRule(id);
      success = "Rule deleted successfully";
      await loadRules();
      setTimeout(() => (success = ""), 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to delete rule";
    }
  }

  onMount(() => {
    loadRules();
  });
</script>

<svelte:head>
  <title>Rules - Fox Daemon Console</title>
</svelte:head>

<div class="page-header">
  <div>
    <h1 class="page-title">Rules</h1>
    <p class="page-subtitle">Configure notification rules for device events</p>
  </div>
  <button class="btn btn-primary" onclick={openCreateModal}>
    Create Rule
  </button>
</div>

{#if success}
  <Alert type="success" message={success} />
{/if}
{#if error && !showModal}
  <Alert type="error" message={error} />
{/if}

{#if loading}
  <Loading message="Loading rules..." />
{:else if rules.length > 0}
  {#each rules as rule}
    <div class="rule-card">
      <div class="rule-header">
        <div style="flex: 1;">
          <div class="rule-title">
            <span>{rule.name}</span>
            <span class="badge badge-{rule.enabled ? 'success' : 'secondary'}">
              {rule.enabled ? "Enabled" : "Disabled"}
            </span>
          </div>

          {#if rule.description}
            <p
              style="margin-top: 0.5rem; font-size: 0.875rem; color: var(--text-secondary);"
            >
              {rule.description}
            </p>
          {/if}

          <div class="rule-meta">
            <div class="rule-meta-item">
              <strong>Trigger:</strong>
              <span class="badge badge-info">
                {getTriggerTypeLabel(rule.trigger_type)}
              </span>
            </div>

            {#if rule.mac_filter}
              <div class="rule-meta-item">
                <strong>MAC Filter:</strong>
                <span class="text-mono">{rule.mac_filter}</span>
              </div>
            {/if}

            {#if rule.notification_channels.length > 0}
              <div class="rule-meta-item">
                <strong>Channels:</strong>
                <span>{rule.notification_channels.length} configured</span>
              </div>
            {/if}
          </div>

          <div
            style="margin-top: 0.5rem; font-size: 0.75rem; color: var(--text-muted);"
          >
            Created: {new Date(rule.created_at).toLocaleString()} · Updated: {new Date(
              rule.updated_at,
            ).toLocaleString()}
          </div>
        </div>

        <div class="rule-actions">
          <button
            class="btn btn-secondary btn-sm"
            onclick={() => openEditModal(rule)}
          >
            Edit
          </button>
          <button
            class="btn btn-danger btn-sm"
            onclick={() => deleteRule(rule.id)}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  {/each}
{:else}
  <div class="card">
    <div class="empty-state">
      <div class="empty-state-title">No rules configured</div>
      <p class="empty-state-text">
        Create your first notification rule to get started
      </p>
      <button class="btn btn-primary" onclick={openCreateModal}
        >Create Your First Rule</button
      >
    </div>
  </div>
{/if}

<Modal
  bind:open={showModal}
  onClose={() => (showModal = false)}
  title={editingRule ? "Edit Rule" : "Create Rule"}
>
  <form onsubmit={handleSubmit}>
    <div class="form-group">
      <label class="form-label" for="name">Rule Name</label>
      <input
        type="text"
        id="name"
        class="form-input"
        bind:value={formData.name}
        required
        placeholder="e.g., Notify on new device"
      />
    </div>

    <div class="form-group">
      <label class="form-label" for="description">Description (optional)</label>
      <textarea
        id="description"
        class="form-textarea"
        bind:value={formData.description}
        placeholder="What does this rule do?"
      ></textarea>
    </div>

    <div class="form-group">
      <label class="form-label" for="trigger_type">Trigger Type</label>
      <select
        id="trigger_type"
        class="form-select"
        bind:value={formData.trigger_type}
        required
      >
        <option value="new_device">New Device</option>
        <option value="device_connected">Device Connected</option>
        <option value="device_disconnected">Device Disconnected</option>
        <option value="device_status_change">Device Status Change</option>
      </select>
    </div>

    <div class="form-group">
      <label class="form-label" for="mac_filter">MAC Filter (optional)</label>
      <input
        type="text"
        id="mac_filter"
        class="form-input"
        bind:value={formData.mac_filter}
        placeholder="aa:bb:cc:dd:ee:ff (leave empty for all devices)"
      />
    </div>

    <div class="form-group">
      <label class="form-label" for="channels">Notification Channels</label>
      <input
        type="text"
        id="channels"
        class="form-input"
        bind:value={formData.notification_channels}
        placeholder="telegram_123456, ntfy_topic (comma-separated)"
      />
      <p class="form-helper">
        Enter channel names as configured in the daemon (e.g., telegram_123456)
      </p>
    </div>

    <div class="checkbox-wrapper">
      <input
        type="checkbox"
        id="enabled"
        class="checkbox"
        bind:checked={formData.enabled}
      />
      <label for="enabled">Rule enabled</label>
    </div>

    {#if error}
      <div style="margin-top: 1rem;">
        <Alert type="error" message={error} />
      </div>
    {/if}

    <div class="modal-footer">
      <button
        type="button"
        class="btn btn-secondary"
        onclick={() => (showModal = false)}
      >
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">
        {editingRule ? "Update" : "Create"} Rule
      </button>
    </div>
  </form>
</Modal>
