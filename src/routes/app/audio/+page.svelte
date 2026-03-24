<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/i18n/index.svelte.js";
  import { configStore, loadConfig } from "$lib/stores/config.svelte.js";
  import {
    ttsStore,
    loadTTSHistory,
    syncWithElevenLabs,
    loadSubscription,
    clearError,
  } from "$lib/stores/tts.svelte.js";
  import { openTTSPanel } from "$lib/stores/ui.svelte.js";
  import CreditsDisplay from "$lib/components/CreditsDisplay.svelte";
  import TTSHistoryList from "$lib/components/TTSHistoryList.svelte";
  import TTSGeneratorPanel from "$lib/components/TTSGeneratorPanel.svelte";
  import ConfigPanel from "$lib/components/ConfigPanel.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import {
    IconPlus,
    IconRefresh,
    IconSettings,
    IconWifi,
    IconWifiOff,
    IconX,
  } from "@tabler/icons-svelte";
  import { openSettings } from "$lib/stores/ui.svelte.js";

  const tr = $derived(t());
  const hasApiKey = $derived(configStore.settings.hasApiKey);
  const isOnline = $derived(ttsStore.isOnline);
  const isSyncing = $derived(ttsStore.isSyncing);
  const error = $derived(ttsStore.error);

  onMount(() => {
    (async () => {
      await loadConfig();
      // Load local history from SQLite (no API call)
      await loadTTSHistory();

      // Only load subscription (quota) on page load - this is fast-changing data
      if (navigator.onLine && configStore.settings.hasApiKey) {
        await loadSubscription();
      }
    })();

    // Listen for online/offline events
    const handleOnline = () => (ttsStore.isOnline = true);
    const handleOffline = () => (ttsStore.isOnline = false);
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  });

  async function handleSync() {
    if (!isOnline || !hasApiKey) return;
    await syncWithElevenLabs();
  }
</script>

<svelte:head><title>{tr.nav.audio} - {tr.app.name}</title></svelte:head>

<div class="page-header">
  <div class="header-left">
    <CreditsDisplay />
    <span
      class="badge connection-status"
      class:badge-active={isOnline}
      class:badge-paused={!isOnline}
    >
      {#if isOnline}
        <IconWifi size={14} />{tr.tts.connectionOnline}
      {:else}
        <IconWifiOff size={14} />
        <span class="status-text">{tr.tts.connectionOffline}</span>
      {/if}
    </span>
  </div>
  <div class="header-actions">
    <button
      class="btn btn-ghost"
      onclick={handleSync}
      disabled={!isOnline || !hasApiKey || isSyncing}
      title={tr.tts.sync}
    >
      <span class:spinning={isSyncing}><IconRefresh size={16} /></span>
      {isSyncing ? tr.tts.syncing : tr.tts.sync}
    </button>
    <button
      class="btn btn-primary"
      onclick={openTTSPanel}
      disabled={!hasApiKey}
      title={tr.tts.generateAudio}
    >
      <IconPlus size={16} />
      {tr.tts.generateAudio}
    </button>
    <button
      class="btn btn-ghost btn-icon"
      onclick={openSettings}
      title={tr.settings.title}><IconSettings size={16} /></button
    >
  </div>
</div>

<div class="page-content">
  {#if error}
    <div class="error-banner">
      <span>{error}</span>
      <button class="close-error" onclick={clearError} aria-label="Dismiss">
        <IconX size={14} />
      </button>
    </div>
  {/if}

  {#if !hasApiKey}
    <div class="no-api-key">
      <p>{tr.settings.apiKeyHint}</p>
      <button class="btn btn-primary" onclick={openSettings}>
        {tr.settings.title}
      </button>
    </div>
  {:else}
    <TTSHistoryList />
  {/if}
</div>

<TTSGeneratorPanel />
<ConfigPanel />
<ConfirmDialog />

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    gap: 12px;
    flex-wrap: wrap;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .connection-status {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--color-text-muted);
  }
  .status-text {
    font-size: 11px;
  }
  .page-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }
  .no-api-key {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 60px 20px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    margin-bottom: 16px;
    background: #fee2e2;
    color: #991b1b;
    border-radius: var(--radius-md);
    font-size: 13px;
  }

  :global([data-theme="dark"]) .error-banner {
    background: #450a0a;
    color: #fecaca;
  }

  .close-error {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    opacity: 0.7;
    padding: 2px;
    display: flex;
  }

  .close-error:hover {
    opacity: 1;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
