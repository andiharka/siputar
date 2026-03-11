<script lang="ts">
  import { uiStore, closeConfirm } from '$lib/stores/ui.svelte.js';

  const dialog = $derived(uiStore.confirmDialog);

  function handleConfirm() {
    dialog.onConfirm();
    closeConfirm();
  }

  function handleCancel() {
    dialog.onCancel?.();
    closeConfirm();
  }
</script>

{#if dialog.open}
  <div class="overlay" role="presentation" onclick={handleCancel} onkeydown={(e) => e.key === 'Escape' && handleCancel()}>
    <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === 'Escape' && handleCancel()}>
      <h3 class="title">{dialog.title}</h3>
      <p class="message">{dialog.message}</p>
      <div class="actions">
        <button class="btn btn-ghost" onclick={handleCancel}>Batal</button>
        <button class="btn btn-danger" onclick={handleConfirm}>Konfirmasi</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed; inset: 0; z-index: 1000;
    background: var(--color-overlay);
    display: flex; align-items: center; justify-content: center;
  }
  .dialog {
    background: var(--color-surface); border-radius: var(--radius-lg);
    padding: 24px; width: 360px; max-width: 90vw;
    box-shadow: var(--shadow-lg);
  }
  .title { font-size: 16px; font-weight: 600; margin-bottom: 8px; }
  .message { color: var(--color-text-muted); font-size: 13px; margin-bottom: 20px; line-height: 1.5; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
