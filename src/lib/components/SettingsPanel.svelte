<script lang="ts">
  import { configStore, updateSettings, saveConfig } from '$lib/stores/config.svelte.js';
  import { t } from '$lib/i18n/index.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { IconCheck, IconX, IconLoader, IconFolder } from '@tabler/icons-svelte';

  const tr = $derived(t());
  const hasApiKey = $derived(configStore.settings.hasApiKey);
  const audioFolder = $derived(configStore.settings.ttsAudioFolder);

  let apiKeyInput = $state('');
  let maskedApiKey = $state('');
  let testStatus = $state<'idle' | 'testing' | 'success' | 'failed'>('idle');
  let testError = $state<string | null>(null);
  let showApiKeyInput = $state(false);
  let isInitialized = $state(false);

  // Verify keychain state on mount only, not on every change
  $effect(() => {
    if (!isInitialized) {
      verifyAndLoadApiKey();
      isInitialized = true;
    }
  });

  async function verifyAndLoadApiKey() {
    try {
      // Check if API key actually exists in keychain
      const exists = await invoke<boolean>('has_api_key');
      
      if (hasApiKey && !exists) {
        // Config says we have a key, but keychain doesn't - sync the state
        console.warn('Config out of sync with keychain - resetting hasApiKey');
        updateSettings({ hasApiKey: false });
        await saveConfig();
        maskedApiKey = '';
      } else if (exists) {
        // Load the masked key for display
        const masked = await invoke<string | null>('get_api_key_masked');
        if (masked) {
          maskedApiKey = masked;
          // Ensure config is in sync
          if (!hasApiKey) {
            updateSettings({ hasApiKey: true });
            await saveConfig();
          }
        }
      } else {
        maskedApiKey = '';
      }
    } catch (e) {
      console.error('Failed to verify API key:', e);
    }
  }

  async function handleRunOnStartup(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    updateSettings({ runOnStartup: checked });
    try {
      if (checked) {
        await invoke('plugin:autostart|enable');
      } else {
        await invoke('plugin:autostart|disable');
      }
    } catch { /* not in Tauri context during dev */ }
  }

  async function handleTestConnection() {
    if (!apiKeyInput.trim()) return;
    
    testStatus = 'testing';
    testError = null;
    try {
      // First test if the key is valid
      const valid = await invoke<boolean>('test_api_key', { key: apiKeyInput.trim() });
      
      if (!valid) {
        testStatus = 'failed';
        testError = 'Invalid API key';
        return;
      }
      
      // Key is valid, now save to keychain
      try {
        await invoke('save_api_key', { key: apiKeyInput.trim() });
      } catch (saveError) {
        testStatus = 'failed';
        testError = `Failed to save: ${saveError instanceof Error ? saveError.message : String(saveError)}`;
        console.error('Failed to save API key to keychain:', saveError);
        return;
      }
      
      // Update config
      updateSettings({ hasApiKey: true });
      await saveConfig();
      
      // Success - hide input
      testStatus = 'success';
      showApiKeyInput = false;
      apiKeyInput = '';
      
      // Wait a moment for keychain to fully persist, then reload
      await new Promise(resolve => setTimeout(resolve, 100));
      await verifyAndLoadApiKey();
      
    } catch (e) {
      testStatus = 'failed';
      testError = e instanceof Error ? e.message : String(e);
      console.error('Test connection failed:', e);
    }
  }

  async function handleRemoveApiKey() {
    try {
      await invoke('delete_api_key');
      updateSettings({ hasApiKey: false });
      await saveConfig();
      maskedApiKey = '';
      apiKeyInput = '';
      showApiKeyInput = false;
    } catch { /* ignore */ }
  }

  async function handleBrowseFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: tr.settings.audioFolder,
      });
      
      if (selected && typeof selected === 'string') {
        updateSettings({ ttsAudioFolder: selected });
      }
    } catch { /* cancelled or error */ }
  }
</script>

<div class="settings">
  <h3 class="section-title">{tr.settings.title}</h3>

  <!-- Theme -->
  <div class="field">
    <span class="field-label">{tr.settings.theme}</span>
    <div class="radio-group">
      {#each [['light', tr.settings.themeLight], ['dark', tr.settings.themeDark], ['auto', tr.settings.themeAuto]] as [val, label]}
        <label class="radio-item">
          <input
            type="radio"
            name="theme"
            value={val}
            checked={configStore.settings.theme === val}
            onchange={() => updateSettings({ theme: val as 'light' | 'dark' | 'auto' })}
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
          type="radio" name="lang" value="id"
          checked={configStore.settings.language === 'id'}
          onchange={() => updateSettings({ language: 'id' })}
        />
        <span>Bahasa Indonesia</span>
      </label>
      <label class="radio-item">
        <input
          type="radio" name="lang" value="en"
          checked={configStore.settings.language === 'en'}
          onchange={() => updateSettings({ language: 'en' })}
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
        <button class="btn btn-ghost btn-sm" onclick={() => showApiKeyInput = true}>
          Change
        </button>
        <button class="btn btn-ghost btn-sm btn-danger-text" onclick={handleRemoveApiKey}>
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
          disabled={!apiKeyInput.trim() || testStatus === 'testing'}
        >
          {#if testStatus === 'testing'}
            <IconLoader size={14} class="spinning" />
          {:else if testStatus === 'success'}
            <IconCheck size={14} />
          {:else if testStatus === 'failed'}
            <IconX size={14} />
          {/if}
          {tr.settings.testConnection}
        </button>
      </div>
      <span class="field-hint">{tr.settings.apiKeyHint}</span>
      
      {#if testStatus === 'success'}
        <div class="status-message success">{tr.settings.testSuccess}</div>
      {:else if testStatus === 'failed'}
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
</div>

<style>
  .settings { padding: 4px 0; }
  .section-title { font-size: 15px; font-weight: 600; margin-bottom: 20px; }
  .subsection-title { font-size: 13px; font-weight: 600; margin-bottom: 16px; color: var(--color-text); }
  .field { margin-bottom: 20px; }
  .field-label { display: block; font-size: 12px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
  .field-hint { display: block; font-size: 11px; color: var(--color-text-muted); margin-top: 6px; }
  .field-row { display: flex; align-items: center; justify-content: space-between; }
  .field-row .field-label { margin-bottom: 0; }
  .radio-group { display: flex; flex-direction: column; gap: 8px; }
  .radio-item { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; }
  .radio-item input { accent-color: var(--color-primary); }
  
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

  :global([data-theme='dark']) .status-message.success {
    background: #166534;
    color: #dcfce7;
  }

  :global([data-theme='dark']) .status-message.error {
    background: #991b1b;
    color: #fee2e2;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
