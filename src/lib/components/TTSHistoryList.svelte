<script lang="ts">
  import { t } from '$lib/i18n/index.svelte.js';
  import { ttsStore } from '$lib/stores/tts.svelte.js';
  import TTSHistoryCard from './TTSHistoryCard.svelte';

  const tr = $derived(t());
  const history = $derived(ttsStore.history);
  const isLoading = $derived(ttsStore.isLoading);
</script>

<div class="history-list">
  {#if isLoading}
    <div class="loading">
      <span class="spinner"></span>
      <span>Loading...</span>
    </div>
  {:else if history.length === 0}
    <div class="empty">
      <p>{tr.tts.noHistory}</p>
    </div>
  {:else}
    <div class="cards">
      {#each history as item (item.id)}
        <TTSHistoryCard {item} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .history-list {
    width: 100%;
  }

  .loading, .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 60px 20px;
    color: var(--color-text-muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
