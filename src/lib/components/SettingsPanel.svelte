<script lang="ts">
  import { configStore, updateSettings, saveConfig } from '$lib/stores/config.svelte.js';
  import { t } from '$lib/i18n/index.svelte.js';
  import { invoke } from '@tauri-apps/api/core';

  const tr = $derived(t());

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
</div>

<style>
  .settings { padding: 4px 0; }
  .section-title { font-size: 15px; font-weight: 600; margin-bottom: 20px; }
  .field { margin-bottom: 20px; }
  .field-label { display: block; font-size: 12px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
  .field-row { display: flex; align-items: center; justify-content: space-between; }
  .field-row .field-label { margin-bottom: 0; }
  .radio-group { display: flex; flex-direction: column; gap: 8px; }
  .radio-item { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; }
  .radio-item input { accent-color: var(--color-primary); }
</style>
