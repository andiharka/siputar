<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { t } from '$lib/i18n/index.svelte.js';
  import { uiStore, closeTTSPanel } from '$lib/stores/ui.svelte.js';
  import { ttsStore, loadVoices, loadModels, generateSpeech } from '$lib/stores/tts.svelte.js';
  import { configStore } from '$lib/stores/config.svelte.js';
  import type { TTSGenerateRequest, ElevenLabsModel } from '$lib/types/index.js';
  import { IconX, IconLoader } from '@tabler/icons-svelte';

  const tr = $derived(t());
  const isOpen = $derived(uiStore.ttsPanelOpen);
  const voices = $derived(ttsStore.voices);
  const models = $derived(ttsStore.models);
  const hasApiKey = $derived(configStore.settings.hasApiKey);

  let text = $state('');
  let voiceId = $state('');
  let voiceName = $state('');
  let modelId = $state('');
  let language = $state('id'); // Default Indonesian
  let stability = $state(0.5);
  let similarityBoost = $state(0.75);
  let speed = $state(1.0);
  let isGenerating = $state(false);
  let error = $state<string | null>(null);

  const selectedModel = $derived<ElevenLabsModel | undefined>(
    models.find(m => m.model_id === modelId)
  );
  
  const maxCharacters = $derived(
    selectedModel?.max_characters_request_subscribed_user || 5000
  );
  
  const charCount = $derived(text.length);
  const charPercent = $derived(Math.min((charCount / maxCharacters) * 100, 100));
  const isOverLimit = $derived(charCount > maxCharacters);
  
  const supportsSettings = $derived(
    selectedModel?.model_id?.includes('eleven_') ?? true
  );

  const languages = [
    { id: 'id', name: 'Indonesian' },
    { id: 'en', name: 'English' },
    { id: 'ms', name: 'Malay' },
    { id: 'jv', name: 'Javanese' },
    { id: 'su', name: 'Sundanese' },
  ];

  onMount(async () => {
    if (hasApiKey && ttsStore.isOnline) {
      await Promise.all([loadVoices(), loadModels()]);
    }
  });

  $effect(() => {
    if (isOpen && voices.length > 0 && !voiceId) {
      voiceId = voices[0].voice_id;
      voiceName = voices[0].name;
    }
    if (isOpen && models.length > 0 && !modelId) {
      modelId = models[0].model_id;
    }
  });

  function handleVoiceChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    voiceId = select.value;
    const voice = voices.find(v => v.voice_id === voiceId);
    voiceName = voice?.name || '';
  }

  async function handleGenerate() {
    if (!text.trim() || !voiceId || !modelId || isOverLimit) return;

    isGenerating = true;
    error = null;

    try {
      const request: TTSGenerateRequest = {
        text: text.trim(),
        voiceId,
        voiceName,
        modelId,
        language,
        stability: supportsSettings ? stability : undefined,
        similarityBoost: supportsSettings ? similarityBoost : undefined,
        speed: supportsSettings ? speed : undefined,
      };

      await generateSpeech(request);
      
      // Reset form
      text = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Generation failed';
    } finally {
      isGenerating = false;
    }
  }

  function handleClose() {
    closeTTSPanel();
    error = null;
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="backdrop"
    role="presentation"
    onclick={handleClose}
    in:fade={{ duration: 160 }}
    out:fade={{ duration: 120 }}
  ></div>
{/if}

<aside class="panel" class:open={isOpen}>
  <div class="panel-header">
    <h3 class="panel-title">{tr.tts.generateAudio}</h3>
    <button class="close-btn" onclick={handleClose} aria-label="Close panel">
      <IconX size={16} />
    </button>
  </div>

  <div class="panel-body">
    <!-- Text Input -->
    <div class="field">
      <label class="field-label" for="tts-text">{tr.tts.text}</label>
      <textarea
        id="tts-text"
        class="input textarea"
        placeholder={tr.tts.textPlaceholder}
        rows="6"
        bind:value={text}
        class:error={isOverLimit}
      ></textarea>
      <div class="char-indicator">
        <div class="char-bar-container">
          <div
            class="char-bar"
            class:warning={charPercent > 80}
            class:error={isOverLimit}
            style="width: {charPercent}%"
          ></div>
        </div>
        <span class="char-count" class:error={isOverLimit}>
          {charCount.toLocaleString()} / {maxCharacters.toLocaleString()} {tr.tts.charLimit}
        </span>
      </div>
    </div>

    <!-- Voice Selection -->
    <div class="field">
      <label class="field-label" for="tts-voice">{tr.tts.voice}</label>
      <select
        id="tts-voice"
        class="select"
        value={voiceId}
        onchange={handleVoiceChange}
      >
        <option value="" disabled>{tr.tts.voicePlaceholder}</option>
        {#each voices as voice}
          <option value={voice.voice_id}>{voice.name}</option>
        {/each}
      </select>
    </div>

    <!-- Model Selection -->
    <div class="field">
      <label class="field-label" for="tts-model">{tr.tts.model}</label>
      <select
        id="tts-model"
        class="select"
        bind:value={modelId}
      >
        <option value="" disabled>{tr.tts.modelPlaceholder}</option>
        {#each models as model}
          <option value={model.model_id}>{model.name}</option>
        {/each}
      </select>
    </div>

    <!-- Language Selection -->
    <div class="field">
      <label class="field-label" for="tts-language">{tr.tts.language}</label>
      <select
        id="tts-language"
        class="select"
        bind:value={language}
      >
        {#each languages as lang}
          <option value={lang.id}>{lang.name}</option>
        {/each}
      </select>
    </div>

    <!-- Settings Sliders (if model supports) -->
    {#if supportsSettings}
      <div class="settings-section">
        <h4 class="settings-title">Voice Settings</h4>

        <!-- Speed -->
        <div class="field">
          <div class="field-row">
            <label class="field-label" for="tts-speed">{tr.tts.speed}</label>
            <span class="field-value">{speed.toFixed(2)}x</span>
          </div>
          <input
            id="tts-speed"
            type="range"
            class="slider"
            min="0.25"
            max="4.0"
            step="0.05"
            bind:value={speed}
          />
          <span class="field-hint">{tr.tts.speedHint}</span>
        </div>

        <!-- Stability -->
        <div class="field">
          <div class="field-row">
            <label class="field-label" for="tts-stability">{tr.tts.stability}</label>
            <span class="field-value">{(stability * 100).toFixed(0)}%</span>
          </div>
          <input
            id="tts-stability"
            type="range"
            class="slider"
            min="0"
            max="1"
            step="0.01"
            bind:value={stability}
          />
          <span class="field-hint">{tr.tts.stabilityHint}</span>
        </div>

        <!-- Similarity Boost -->
        <div class="field">
          <div class="field-row">
            <label class="field-label" for="tts-similarity">{tr.tts.similarity}</label>
            <span class="field-value">{(similarityBoost * 100).toFixed(0)}%</span>
          </div>
          <input
            id="tts-similarity"
            type="range"
            class="slider"
            min="0"
            max="1"
            step="0.01"
            bind:value={similarityBoost}
          />
          <span class="field-hint">{tr.tts.similarityHint}</span>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-message">{error}</div>
    {/if}
  </div>

  <div class="panel-footer">
    <button class="btn btn-ghost" onclick={handleClose}>
      {tr.actions.cancel}
    </button>
    <button
      class="btn btn-primary"
      onclick={handleGenerate}
      disabled={!text.trim() || !voiceId || !modelId || isOverLimit || isGenerating}
    >
      {#if isGenerating}
        <IconLoader size={16} class="spinning" />
      {/if}
      {tr.actions.generate}
    </button>
  </div>
</aside>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 199;
    background: rgba(0, 0, 0, 0.4);
  }

  .panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 380px;
    max-width: 100vw;
    z-index: 200;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform 0.2s ease;
  }

  .panel.open {
    transform: translateX(0);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .panel-title {
    font-size: 16px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }

  .close-btn:hover {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .field {
    margin-bottom: 16px;
  }

  .field-label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 6px;
  }

  .field-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }

  .field-value {
    font-size: 12px;
    color: var(--color-text);
    font-weight: 500;
  }

  .field-hint {
    display: block;
    font-size: 11px;
    color: var(--color-text-muted);
    margin-top: 4px;
  }

  .textarea {
    resize: vertical;
    min-height: 100px;
    font-family: inherit;
  }

  .textarea.error {
    border-color: var(--color-danger);
  }

  .char-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }

  .char-bar-container {
    flex: 1;
    height: 4px;
    background: var(--color-surface-2);
    border-radius: 2px;
    overflow: hidden;
  }

  .char-bar {
    height: 100%;
    background: var(--color-primary);
    border-radius: 2px;
    transition: width 0.2s ease;
  }

  .char-bar.warning {
    background: var(--color-warning);
  }

  .char-bar.error {
    background: var(--color-danger);
  }

  .char-count {
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .char-count.error {
    color: var(--color-danger);
  }

  .settings-section {
    margin-top: 24px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border);
  }

  .settings-title {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--color-text);
  }

  .slider {
    width: 100%;
    height: 4px;
    appearance: none;
    background: var(--color-surface-2);
    border-radius: 2px;
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    background: var(--color-primary);
    border-radius: 50%;
    cursor: pointer;
  }

  .error-message {
    background: var(--color-danger);
    color: white;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    margin-top: 12px;
  }

  .panel-footer {
    display: flex;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid var(--color-border);
    justify-content: flex-end;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
