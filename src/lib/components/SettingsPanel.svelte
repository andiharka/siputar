<script lang="ts">
  import {
    configStore,
    updateSettings,
    saveConfig,
  } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    IconCheck,
    IconX,
    IconLoader,
    IconFolder,
    IconTrash,
    IconFolderOpen,
    IconChevronDown,
    IconChevronUp,
  } from "@tabler/icons-svelte";
  import { onMount } from "svelte";

  interface ActivityLogEntry {
    ts: string;
    cat: string;
    act: string;
    data: Record<string, unknown>;
  }

  const tr = $derived(t());
  const hasApiKey = $derived(configStore.settings.hasApiKey);
  const audioFolder = $derived(configStore.settings.ttsAudioFolder);

  let apiKeyInput = $state("");
  let maskedApiKey = $state("");
  let testStatus = $state<"idle" | "testing" | "success" | "failed">("idle");
  let testError = $state<string | null>(null);
  let showApiKeyInput = $state(false);

  // Activity log state
  let activityLogs = $state<ActivityLogEntry[]>([]);
  let showActivityLog = $state(false);
  let logFilter = $state<"all" | "schedule" | "voice" | "playback">("all");
  let loadingLogs = $state(false);

  const filteredLogs = $derived(
    logFilter === "all"
      ? activityLogs
      : activityLogs.filter((log) => log.cat === logFilter),
  );

  // Verify keychain state on mount only - using onMount instead of $effect
  onMount(() => {
    verifyAndLoadApiKey();
  });

  async function verifyAndLoadApiKey() {
    try {
      // Check if API key actually exists in keychain
      const exists = await invoke<boolean>("has_api_key");

      if (hasApiKey && !exists) {
        // Config says we have a key, but keychain doesn't - sync the state
        console.warn("Config out of sync with keychain - resetting hasApiKey");
        updateSettings({ hasApiKey: false });
        await saveConfig();
        maskedApiKey = "";
      } else if (exists) {
        // Load the masked key for display
        const masked = await invoke<string | null>("get_api_key_masked");
        if (masked) {
          maskedApiKey = masked;
          // Ensure config is in sync
          if (!hasApiKey) {
            updateSettings({ hasApiKey: true });
            await saveConfig();
          }
        }
      } else {
        maskedApiKey = "";
      }
    } catch (e) {
      console.error("Failed to verify API key:", e);
    }
  }

  async function handleRunOnStartup(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    updateSettings({ runOnStartup: checked });
    try {
      if (checked) {
        await invoke("plugin:autostart|enable");
      } else {
        await invoke("plugin:autostart|disable");
      }
    } catch {
      /* not in Tauri context during dev */
    }
  }

  async function handleTestConnection() {
    if (!apiKeyInput.trim()) return;

    testStatus = "testing";
    testError = null;
    try {
      // First test if the key is valid
      const valid = await invoke<boolean>("test_api_key", {
        key: apiKeyInput.trim(),
      });

      if (!valid) {
        testStatus = "failed";
        testError = "Invalid API key";
        return;
      }

      // Key is valid, now save to keychain
      try {
        await invoke("save_api_key", { key: apiKeyInput.trim() });
      } catch (saveError) {
        testStatus = "failed";
        testError = `Failed to save: ${saveError instanceof Error ? saveError.message : String(saveError)}`;
        console.error("Failed to save API key to keychain:", saveError);
        return;
      }

      // Update config
      updateSettings({ hasApiKey: true });
      await saveConfig();

      // Success - hide input
      testStatus = "success";
      showApiKeyInput = false;
      apiKeyInput = "";

      // Wait a moment for keychain to fully persist, then reload
      await new Promise((resolve) => setTimeout(resolve, 100));
      await verifyAndLoadApiKey();
    } catch (e) {
      testStatus = "failed";
      testError = e instanceof Error ? e.message : String(e);
      console.error("Test connection failed:", e);
    }
  }

  async function handleRemoveApiKey() {
    try {
      await invoke("delete_api_key");
      updateSettings({ hasApiKey: false });
      await saveConfig();
      maskedApiKey = "";
      apiKeyInput = "";
      showApiKeyInput = false;
    } catch {
      /* ignore */
    }
  }

  async function handleBrowseFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: tr.settings.audioFolder,
      });

      if (selected && typeof selected === "string") {
        updateSettings({ ttsAudioFolder: selected });
      }
    } catch {
      /* cancelled or error */
    }
  }

  async function loadActivityLogs() {
    loadingLogs = true;
    try {
      activityLogs = await invoke<ActivityLogEntry[]>("get_activity_logs", {
        limit: 100,
        offset: 0,
      });
    } catch (e) {
      console.error("Failed to load activity logs:", e);
    } finally {
      loadingLogs = false;
    }
  }

  async function handleClearLogs() {
    if (!confirm(tr.settings.activityLog.confirmClear)) return;
    try {
      await invoke("clear_activity_logs");
      activityLogs = [];
    } catch (e) {
      console.error("Failed to clear logs:", e);
    }
  }

  async function handleOpenLogFolder() {
    try {
      await invoke("open_log_folder");
    } catch (e) {
      console.error("Failed to open log folder:", e);
    }
  }

  function toggleActivityLog() {
    showActivityLog = !showActivityLog;
    if (showActivityLog && activityLogs.length === 0) {
      loadActivityLogs();
    }
  }

  function formatLogTime(ts: string): string {
    const date = new Date(ts);
    return date.toLocaleString();
  }

  function formatLogAction(cat: string, act: string): string {
    const labels: Record<string, Record<string, string>> = {
      schedule: {
        create: tr.settings.activityLog.scheduleCreate,
        update: tr.settings.activityLog.scheduleUpdate,
        delete: tr.settings.activityLog.scheduleDelete,
      },
      voice: {
        generate: tr.settings.activityLog.voiceGenerate,
        delete: tr.settings.activityLog.voiceDelete,
        play: tr.settings.activityLog.voicePlay,
      },
      playback: {
        running: tr.settings.activityLog.playbackRunning,
        paused: tr.settings.activityLog.playbackPaused,
        stopped: tr.settings.activityLog.playbackStopped,
      },
    };
    return labels[cat]?.[act] || `${cat}:${act}`;
  }

  function getCategoryColor(cat: string): string {
    switch (cat) {
      case "schedule":
        return "var(--color-primary)";
      case "voice":
        return "#10b981";
      case "playback":
        return "#f59e0b";
      default:
        return "var(--color-text-muted)";
    }
  }
</script>

<div class="settings">
  <!-- Theme -->
  <div class="field">
    <span class="field-label">{tr.settings.theme}</span>
    <div class="radio-group">
      {#each [["light", tr.settings.themeLight], ["dark", tr.settings.themeDark], ["auto", tr.settings.themeAuto]] as [val, label]}
        <label class="radio-item">
          <input
            type="radio"
            name="theme"
            value={val}
            checked={configStore.settings.theme === val}
            onchange={() =>
              updateSettings({ theme: val as "light" | "dark" | "auto" })}
          />
          <span>{label}</span>
        </label>
      {/each}
    </div>
  </div>

  <!-- Language -->
  <div class="field">
    <span class="field-label">{tr.settings.language}</span>
    <div class="radio-group">
      <label class="radio-item">
        <input
          type="radio"
          name="lang"
          value="id"
          checked={configStore.settings.language === "id"}
          onchange={() => updateSettings({ language: "id" })}
        />
        <span>Bahasa Indonesia</span>
      </label>
      <label class="radio-item">
        <input
          type="radio"
          name="lang"
          value="en"
          checked={configStore.settings.language === "en"}
          onchange={() => updateSettings({ language: "en" })}
        />
        <span>English</span>
      </label>
    </div>
  </div>

  <!-- Run on startup -->
  <div class="field field-row">
    <span class="field-label">{tr.settings.runOnStartup}</span>
    <label class="toggle">
      <input
        type="checkbox"
        checked={configStore.settings.runOnStartup}
        onchange={handleRunOnStartup}
      />
      <span class="toggle-slider"></span>
    </label>
  </div>

  <hr class="divider" />

  <!-- ElevenLabs API Key -->
  <h4 class="subsection-title">ElevenLabs TTS</h4>

  <div class="field">
    <span class="field-label">{tr.settings.apiKey}</span>

    {#if hasApiKey && !showApiKeyInput}
      <div class="api-key-display">
        <span class="masked-key">{maskedApiKey}</span>
        <button
          class="btn btn-ghost btn-sm"
          onclick={() => (showApiKeyInput = true)}
        >
          Change
        </button>
        <button
          class="btn btn-ghost btn-sm btn-danger-text"
          onclick={handleRemoveApiKey}
        >
          Remove
        </button>
      </div>
    {:else}
      <div class="api-key-input-group">
        <input
          type="password"
          class="input"
          placeholder={tr.settings.apiKeyPlaceholder}
          bind:value={apiKeyInput}
        />
        <button
          class="btn btn-primary"
          onclick={handleTestConnection}
          disabled={!apiKeyInput.trim() || testStatus === "testing"}
        >
          {#if testStatus === "testing"}
            <IconLoader size={14} class="spinning" />
          {:else if testStatus === "success"}
            <IconCheck size={14} />
          {:else if testStatus === "failed"}
            <IconX size={14} />
          {/if}
          {tr.settings.testConnection}
        </button>
      </div>
      <span class="field-hint">{tr.settings.apiKeyHint}</span>

      {#if testStatus === "success"}
        <div class="status-message success">{tr.settings.testSuccess}</div>
      {:else if testStatus === "failed"}
        <div class="status-message error">
          {tr.settings.testFailed}
          {#if testError}
            <span class="error-detail">: {testError}</span>
          {/if}
        </div>
      {/if}
    {/if}
  </div>

  <!-- Audio Folder -->
  <div class="field">
    <span class="field-label">{tr.settings.audioFolder}</span>
    <div class="folder-input-group">
      <input
        type="text"
        class="input"
        readonly
        value={audioFolder}
        placeholder="Select a folder..."
      />
      <button class="btn btn-ghost btn-icon" onclick={handleBrowseFolder}>
        <IconFolder size={16} />
      </button>
    </div>
    <span class="field-hint">{tr.settings.audioFolderHint}</span>
  </div>

  <hr class="divider" />

  <!-- Activity Log -->
  <div class="activity-log-section">
    <button class="activity-log-header" onclick={toggleActivityLog}>
      <span class="subsection-title">{tr.settings.activityLog.title}</span>
      {#if showActivityLog}
        <IconChevronUp size={16} />
      {:else}
        <IconChevronDown size={16} />
      {/if}
    </button>

    {#if showActivityLog}
      <div class="activity-log-content">
        <div class="activity-log-toolbar">
          <div class="filter-group">
            <select class="select-small" bind:value={logFilter}>
              <option value="all">{tr.settings.activityLog.filterAll}</option>
              <option value="schedule"
                >{tr.settings.activityLog.filterSchedule}</option
              >
              <option value="voice"
                >{tr.settings.activityLog.filterVoice}</option
              >
              <option value="playback"
                >{tr.settings.activityLog.filterPlayback}</option
              >
            </select>
          </div>
          <div class="action-buttons">
            <button
              class="btn btn-ghost btn-sm"
              onclick={loadActivityLogs}
              disabled={loadingLogs}
            >
              {#if loadingLogs}
                <IconLoader size={14} class="spinning" />
              {/if}
              {tr.settings.activityLog.refresh}
            </button>
          </div>
          <div>
            <button class="btn btn-ghost btn-sm" onclick={handleOpenLogFolder}>
              <IconFolderOpen size={14} />
            </button>
            <button
              class="btn btn-ghost btn-sm btn-danger-text"
              onclick={handleClearLogs}
            >
              <IconTrash size={14} />
            </button>
          </div>
        </div>

        <div class="activity-log-list">
          {#if filteredLogs.length === 0}
            <div class="empty-state">{tr.settings.activityLog.empty}</div>
          {:else}
            {#each filteredLogs as log}
              <div class="log-entry">
                <span class="log-time">{formatLogTime(log.ts)}</span>
                <span
                  class="log-category"
                  style="color: {getCategoryColor(log.cat)}">{log.cat}</span
                >
                <span class="log-action"
                  >{formatLogAction(log.cat, log.act)}</span
                >
                {#if log.data && Object.keys(log.data).length > 0}
                  <span class="log-data">{JSON.stringify(log.data)}</span>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings {
    padding: 4px 0;
  }
  .section-title {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 20px;
  }
  .subsection-title {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--color-text);
  }
  .field {
    margin-bottom: 20px;
  }
  .field-label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 8px;
  }
  .field-hint {
    display: block;
    font-size: 11px;
    color: var(--color-text-muted);
    margin-top: 6px;
  }
  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .field-row .field-label {
    margin-bottom: 0;
  }
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .radio-item {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
  }
  .radio-item input {
    accent-color: var(--color-primary);
  }

  .divider {
    border: none;
    border-top: 1px solid var(--color-border);
    margin: 24px 0;
  }

  .api-key-display {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .masked-key {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-text);
    background: var(--color-surface-2);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
  }

  .api-key-input-group {
    display: flex;
    gap: 8px;
  }

  .api-key-input-group .input {
    flex: 1;
  }

  .folder-input-group {
    display: flex;
    gap: 8px;
  }

  .folder-input-group .input {
    flex: 1;
  }

  .btn-sm {
    padding: 4px 8px;
    font-size: 11px;
  }

  .btn-danger-text {
    color: var(--color-danger);
  }

  .btn-danger-text:hover {
    background: var(--color-danger);
    color: white;
  }

  .status-message {
    font-size: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    margin-top: 8px;
  }

  .status-message.success {
    background: #dcfce7;
    color: #166534;
  }

  .status-message.error {
    background: #fee2e2;
    color: #991b1b;
  }

  :global([data-theme="dark"]) .status-message.success {
    background: #166534;
    color: #dcfce7;
  }

  :global([data-theme="dark"]) .status-message.error {
    background: #991b1b;
    color: #fee2e2;
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

  /* Activity Log Styles */
  .activity-log-section {
    margin-top: 8px;
  }

  .activity-log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: var(--color-text);
  }

  .activity-log-header .subsection-title {
    margin-bottom: 0;
  }

  .activity-log-content {
    margin-top: 16px;
  }

  .activity-log-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    gap: 8px;
  }

  .filter-group {
    display: flex;
    gap: 8px;
  }

  .select-small {
    font-size: 12px;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
  }

  .action-buttons {
    display: flex;
    gap: 4px;
  }

  .activity-log-list {
    max-height: 300px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-3);
  }

  .empty-state {
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .log-entry {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    font-size: 12px;
  }

  .log-entry:last-child {
    border-bottom: none;
  }

  .log-time {
    color: var(--color-text-muted);
    font-size: 11px;
    white-space: nowrap;
  }

  .log-category {
    font-weight: 600;
    text-transform: uppercase;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.1);
  }

  :global([data-theme="dark"]) .log-category {
    background: rgba(255, 255, 255, 0.1);
  }

  .log-action {
    color: var(--color-text);
  }

  .log-data {
    color: var(--color-text-muted);
    font-family: monospace;
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 200px;
  }
</style>
