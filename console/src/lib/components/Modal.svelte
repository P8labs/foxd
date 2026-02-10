<script lang="ts">
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
    title: string;
    children: import("svelte").Snippet;
  }

  let { open = $bindable(), onClose, title, children }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      onClose();
    }
  }

  onMount(() => {
    if (browser) {
      document.addEventListener("keydown", handleKeydown);
    }
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("keydown", handleKeydown);
    }
  });
</script>

{#if open}
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <h3 class="modal-title">{title}</h3>
        <button type="button" class="modal-close" onclick={onClose}
          >&times;</button
        >
      </div>
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}
