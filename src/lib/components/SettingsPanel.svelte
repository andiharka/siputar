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
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import {
    IconRefresh,
    IconDownload,
    IconArrowRight,
    IconSun,
    IconMoon,
    IconDeviceDesktop,
    IconLanguage,
  } from "@tabler/icons-svelte";

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

  // Update state
  type UpdateStatus =
    | "idle"
    | "checking"
    | "up-to-date"
    | "available"
    | "downloading"
    | "installing"
    | "restart-required"
    | "failed";
  let updateStatus = $state<UpdateStatus>("idle");
  let updateVersion = $state<string | null>(null);
  let updateNotes = $state<string | null>(null);
  let downloadProgress = $state(0);
  let downloadTotal = $state(0);
  let updateError = $state<string | null>(null);
  let pendingUpdate = $state<Awaited<ReturnType<typeof check>> | null>(null);
  let appVersion = $state<string>("...");

  const filteredLogs = $derived(
    logFilter === "all"
      ? activityLogs
      : activityLogs.filter((log) => log.cat === logFilter),
  );

  onMount(() => {
    verifyAndLoadApiKey();
    loadAppVersion();
  });

  async function loadAppVersion() {
    try {
      appVersion = await invoke<string>("get_app_version");
    } catch {
      appVersion = "?";
    }
  }

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
      await saveConfig();
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
        await saveConfig();
      }
    } catch (e) {
      console.error("Failed to open dialog:", e);
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
  async function handleCheckForUpdates() {
    updateStatus = "checking";
    updateError = null;
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateVersion = update.version;
        updateNotes = update.body ?? null;
        updateStatus = "available";
      } else {
        updateStatus = "up-to-date";
      }
    } catch (e) {
      updateStatus = "failed";
      updateError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleInstallUpdate() {
    if (!pendingUpdate) return;
    updateStatus = "downloading";
    downloadProgress = 0;
    downloadTotal = 0;
    try {
      await pendingUpdate.downloadAndInstall((event) => {
        if (event.event === "Started") {
          downloadTotal = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloadProgress += event.data.chunkLength;
        } else if (event.event === "Finished") {
          updateStatus = "restart-required";
        }
      });
      updateStatus = "restart-required";
    } catch (e) {
      updateStatus = "failed";
      updateError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleRelaunch() {
    await relaunch();
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  }
</script>

<div class="settings">
  <!-- About & Updates -->
  <div class="field">
    <div class="field-row" style="margin-bottom: 12px;">
      <span class="field-label" style="margin-bottom: 0;"
        >{tr.settings.update.currentVersion}</span
      >
      <span class="version-badge">v{appVersion}</span>
    </div>

    {#if updateStatus === "idle" || updateStatus === "up-to-date" || updateStatus === "failed"}
      <button
        class="btn btn-ghost btn-sm"
        onclick={handleCheckForUpdates}
        style="width: 100%; padding: .5rem .75rem;"
      >
        <IconRefresh size={14} />
        {tr.settings.update.checkForUpdates}
      </button>
      {#if updateStatus === "up-to-date"}
        <div class="update-message update-success">
          ✓ {tr.settings.update.upToDate}
        </div>
      {/if}
      {#if updateStatus === "failed"}
        <div class="update-message update-error">
          {tr.settings.update.failed}{updateError ? `: ${updateError}` : ""}
        </div>
      {/if}
    {:else if updateStatus === "checking"}
      <div class="update-message">
        <IconLoader size={14} class="spinning" />
        {tr.settings.update.checking}
      </div>
    {:else if updateStatus === "available"}
      <div class="update-available">
        <div class="update-version">
          🎉 {tr.settings.update.available}: <strong>v{updateVersion}</strong>
        </div>
        {#if updateNotes}
          <details class="release-notes">
            <summary>{tr.settings.update.releaseNotes}</summary>
            <pre class="release-notes-body">{updateNotes}</pre>
          </details>
        {/if}
        <button
          class="btn btn-primary btn-sm"
          onclick={handleInstallUpdate}
          style="width: 100%; margin-top: 8px;"
        >
          <IconDownload size={14} />
          {tr.settings.update.downloadAndInstall}
        </button>
      </div>
    {:else if updateStatus === "downloading"}
      <div class="update-progress">
        <div class="progress-label">
          <span>{tr.settings.update.downloading}</span>
          {#if downloadTotal > 0}
            <span
              >{formatBytes(downloadProgress)} / {formatBytes(
                downloadTotal,
              )}</span
            >
          {/if}
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {downloadTotal > 0
              ? ((downloadProgress / downloadTotal) * 100).toFixed(0)
              : 0}%"
          ></div>
        </div>
      </div>
    {:else if updateStatus === "restart-required"}
      <div class="update-message update-success">
        ✓ {tr.settings.update.restartRequired}
      </div>
      <button
        class="btn btn-primary btn-sm"
        onclick={handleRelaunch}
        style="width: 100%; margin-top: 8px;"
      >
        <IconArrowRight size={14} />
        {tr.settings.update.restartNow}
      </button>
    {/if}
  </div>

  <!-- Theme -->
  <div class="field">
    <span class="field-label">{tr.settings.theme}</span>
    <div class="btn-group" style="width: 100%;">
      <button
        class="btn {configStore.settings.theme === 'light'
          ? 'btn-primary'
          : 'btn-ghost'} btn-theme"
        onclick={async () => {
          updateSettings({ theme: "light" });
          await saveConfig();
        }}
      >
        <IconSun size={16} />
        <div>
          {tr.settings.themeLight}
        </div>
      </button>
      <button
        class="btn {configStore.settings.theme === 'dark'
          ? 'btn-primary'
          : 'btn-ghost'} btn-theme"
        onclick={async () => {
          updateSettings({ theme: "dark" });
          await saveConfig();
        }}
      >
        <IconMoon size={16} />
        {tr.settings.themeDark}
      </button>
      <button
        class="btn {configStore.settings.theme === 'auto'
          ? 'btn-primary'
          : 'btn-ghost'} btn-theme"
        onclick={async () => {
          updateSettings({ theme: "auto" });
          await saveConfig();
        }}
      >
        <IconDeviceDesktop size={16} />
        {tr.settings.themeAuto}
      </button>
    </div>
  </div>

  <!-- Language -->
  <div class="field">
    <span class="field-label">{tr.settings.language}</span>
    <div class="btn-group" style="width: 100%;">
      <button
        class="btn {configStore.settings.language === 'id'
          ? 'btn-primary'
          : 'btn-ghost'}"
        style="flex: 1; justify-content: center;"
        onclick={async () => {
          updateSettings({ language: "id" });
          await saveConfig();
        }}
      >
        <IconLanguage size={16} />
        Indonesia
      </button>
      <button
        class="btn {configStore.settings.language === 'en'
          ? 'btn-primary'
          : 'btn-ghost'}"
        style="flex: 1; justify-content: center;"
        onclick={async () => {
          updateSettings({ language: "en" });
          await saveConfig();
        }}
      >
        <IconLanguage size={16} />
        English
      </button>
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

  <!-- ElevenLabs Collection ID -->
  <div class="field">
    <span class="field-label">ElevenLabs Collection ID</span>
    <input
      type="text"
      class="input"
      placeholder="Leave empty to show all voices"
      value={configStore.settings.elevenLabsCollectionId}
      onchange={async (e) => {
        updateSettings({
          elevenLabsCollectionId: (e.target as HTMLInputElement).value,
        });
        await saveConfig();
      }}
    />
    <span class="field-hint"
      >Filter voices by collection ID in the generation panel.</span
    >
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
  .btn-theme {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0.5rem 0.75rem;
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

  .version-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 100px;
    background: var(
      --color-primary-alpha,
      rgba(var(--color-primary-rgb, 99, 102, 241), 0.12)
    );
    color: var(--color-primary);
  }

  .update-message {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    padding: 8px 0 4px;
    color: var(--color-text-muted);
  }

  .update-success {
    color: #16a34a;
  }
  :global([data-theme="dark"]) .update-success {
    color: #4ade80;
  }

  .update-error {
    color: #dc2626;
    font-size: 11px;
    word-break: break-word;
  }
  :global([data-theme="dark"]) .update-error {
    color: #f87171;
  }

  .update-available {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 12px;
    background: var(--color-surface-alt, var(--color-surface));
  }

  .update-version {
    font-size: 13px;
    margin-bottom: 6px;
  }

  .release-notes {
    margin-top: 6px;
    font-size: 11px;
  }

  .release-notes summary {
    cursor: pointer;
    color: var(--color-text-muted);
    margin-bottom: 4px;
  }

  .release-notes-body {
    margin: 4px 0 0;
    max-height: 120px;
    overflow-y: auto;
    white-space: pre-wrap;
    font-size: 11px;
    color: var(--color-text-muted);
    font-family: inherit;
  }

  .update-progress {
    padding: 4px 0;
  }

  .progress-label {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--color-text-muted);
    margin-bottom: 6px;
  }

  .progress-bar {
    height: 6px;
    background: var(--color-border);
    border-radius: 100px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    border-radius: 100px;
    transition: width 0.2s ease;
  }
</style>
