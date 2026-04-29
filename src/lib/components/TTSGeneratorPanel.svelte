<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { t } from "$lib/i18n/index.svelte.js";
  import { uiStore, closeTTSPanel } from "$lib/stores/ui.svelte.js";
  import {
    ttsStore,
    loadVoices,
    loadModels,
    generateSpeech,
  } from "$lib/stores/tts.svelte.js";
  import { configStore, updateSettings, saveConfig } from "$lib/stores/config.svelte.js";
  import type {
    TTSGenerateRequest,
    ElevenLabsModel,
  } from "$lib/types/index.js";
  import { IconX, IconLoader, IconRefresh, IconPlayerPlay, IconPlayerPause } from "@tabler/icons-svelte";

  const tr = $derived(t());
  const isOpen = $derived(uiStore.ttsPanelOpen);
  const voices = $derived(ttsStore.voices);
  const models = $derived(ttsStore.models);
  const hasApiKey = $derived(configStore.settings.hasApiKey);

  let name = $state("");
  let text = $state("");
  let voiceId = $state("");
  let voiceName = $state("");
  let modelId = $state("");
  let language = $state("id"); // Default Indonesian
  let stability = $state(0.5);
  let similarityBoost = $state(0.75);
  let speed = $state(1.0);
  let isGenerating = $state(false);
  let isRefreshing = $state(false);
  let error = $state<string | null>(null);

  // Audio preview state
  let currentAudio = $state<HTMLAudioElement | null>(null);
  let isPlayingPreview = $state(false);

  function getLanguageEmoji(languageName: string): string {
    if (!languageName) return '🌍';
    const map: Record<string, string> = {
        'english': '🇺🇸', 'en': '🇺🇸', 'en-us': '🇺🇸', 'en-gb': '🇬🇧', 'american': '🇺🇸', 'british': '🇬🇧', 'australian': '🇦🇺',
        'indonesian': '🇮🇩', 'id': '🇮🇩',
        'malay': '🇲🇾', 'ms': '🇲🇾',
        'japanese': '🇯🇵', 'ja': '🇯🇵',
        'chinese': '🇨🇳', 'zh': '🇨🇳',
        'korean': '🇰🇷', 'ko': '🇰🇷',
        'spanish': '🇪🇸', 'es': '🇪🇸',
        'french': '🇫🇷', 'fr': '🇫🇷',
        'german': '🇩🇪', 'de': '🇩🇪',
        'italian': '🇮🇹', 'it': '🇮🇹',
        'portuguese': '🇵🇹', 'pt': '🇵🇹',
        'russian': '🇷🇺', 'ru': '🇷🇺',
        'hindi': '🇮🇳', 'hi': '🇮🇳',
        'arabic': '🇸🇦', 'ar': '🇸🇦',
    };
    const key = languageName.toLowerCase();
    for (const [k, v] of Object.entries(map)) {
        if (key.includes(k) || key === k) return v;
    }
    return '🌍';
  }

  const languageNames: Record<string, string> = {
    'en': 'English', 'id': 'Indonesian', 'ms': 'Malay', 'jv': 'Javanese', 'su': 'Sundanese',
    'ja': 'Japanese', 'zh': 'Chinese', 'ko': 'Korean', 'es': 'Spanish', 'fr': 'French',
    'de': 'German', 'it': 'Italian', 'pt': 'Portuguese', 'ru': 'Russian', 'hi': 'Hindi',
    'ar': 'Arabic', 'nl': 'Dutch', 'tr': 'Turkish', 'pl': 'Polish', 'sv': 'Swedish',
    'bg': 'Bulgarian', 'ro': 'Romanian', 'cs': 'Czech', 'el': 'Greek', 'fi': 'Finnish',
    'hr': 'Croatian', 'da': 'Danish', 'ta': 'Tamil', 'uk': 'Ukrainian', 'sk': 'Slovak',
    'hu': 'Hungarian', 'no': 'Norwegian', 'vi': 'Vietnamese'
  };

  function getLanguageLongName(code: string): string {
    if (!code) return '';
    return languageNames[code.toLowerCase()] || code;
  }

  function playPreview(url?: string) {
    if (!url) return;
    if (currentAudio) {
      currentAudio.pause();
      if (currentAudio.src === url) {
        currentAudio = null;
        isPlayingPreview = false;
        return;
      }
    }
    currentAudio = new Audio(url);
    isPlayingPreview = true;
    currentAudio.onended = () => {
      isPlayingPreview = false;
      currentAudio = null;
    };
    currentAudio.play().catch(() => {
      isPlayingPreview = false;
      currentAudio = null;
    });
  }

  const filteredVoices = $derived(
    configStore.settings.elevenLabsCollectionId 
      ? voices.filter(v => v.collection_ids?.includes(configStore.settings.elevenLabsCollectionId))
      : voices
  );

  const selectedVoice = $derived(filteredVoices.find(v => v.voice_id === voiceId));

  const PINNED_MODEL_ID = 'eleven_v3';

  const availableModels = $derived.by(() => {
    const pinnedModel = models.find(m => m.model_id === PINNED_MODEL_ID);
    const voiceModels = selectedVoice?.high_quality_base_model_ids?.length
      ? models.filter(m => selectedVoice.high_quality_base_model_ids?.includes(m.model_id) && m.model_id !== PINNED_MODEL_ID)
      : models.filter(m => m.model_id !== PINNED_MODEL_ID);
    return pinnedModel ? [pinnedModel, ...voiceModels] : voiceModels;
  });

  const selectedModel = $derived<ElevenLabsModel | undefined>(
    availableModels.find((m) => m.model_id === modelId),
  );

  const maxCharacters = $derived(
    selectedModel?.max_characters_request_subscribed_user || 5000,
  );

  const charCount = $derived(text.length);
  const charPercent = $derived(
    Math.min((charCount / maxCharacters) * 100, 100),
  );
  const isOverLimit = $derived(charCount > maxCharacters);

  // All current ElevenLabs TTS models support voice_settings (stability,
  // similarity_boost, speed). Only exclude models that are known not to
  // support them (none currently — so default to true).
  const supportsSettings = $derived(
    selectedModel ? selectedModel.can_do_text_to_speech : true,
  );

  const availableLanguages = $derived.by(() => {
    // Collect distinct base language codes from the voice metadata
    const seen = new Set<string>();
    const langs: { id: string; label: string }[] = [];

    // Always pin Indonesian at the top
    seen.add('id');
    langs.push({ id: 'id', label: `${getLanguageEmoji('indonesian')} Indonesian` });

    if (selectedVoice) {
      const sources: string[] = [];
      if (selectedVoice.verified_languages?.length) {
        selectedVoice.verified_languages.forEach(v => sources.push(v.language));
      } else if (selectedVoice.labels?.language) {
        sources.push(selectedVoice.labels.language);
      }

      for (const code of sources) {
        const base = code.split('-')[0].toLowerCase();
        if (seen.has(base)) continue;
        seen.add(base);
        const longName = getLanguageLongName(base);
        langs.push({ id: base, label: `${getLanguageEmoji(base)} ${longName}` });
      }
    }

    return langs;
  });

  onMount(async () => {
    if (hasApiKey && ttsStore.isOnline) {
      await Promise.all([loadVoices(), loadModels()]);
    }
  });

  $effect(() => {
    if (isOpen && filteredVoices.length > 0 && !voiceId) {
      voiceId = configStore.settings.lastSelectedVoiceId || filteredVoices[0].voice_id;
      const voice = filteredVoices.find((v) => v.voice_id === voiceId) || filteredVoices[0];
      voiceId = voice.voice_id;
      voiceName = voice.name;
    }
    if (isOpen && availableModels.length > 0 && !modelId) {
      modelId = configStore.settings.lastSelectedModelId || availableModels[0].model_id;
      const model = availableModels.find((m) => m.model_id === modelId) || availableModels[0];
      modelId = model.model_id;
    }
    if (isOpen && !language) {
      language = configStore.settings.lastSelectedLanguage || availableLanguages[0]?.id || "id";
    }
  });

  $effect(() => {
    if (isOpen) {
      console.log(`[ElevenLabs] Filtering voices by collection ID: '${configStore.settings.elevenLabsCollectionId}'`);
      console.log(`[ElevenLabs] Found ${filteredVoices.length} voices (out of ${voices.length})`);
    }
  });

  $effect(() => {
    if (isOpen) {
      let changed = false;
      if (voiceId && configStore.settings.lastSelectedVoiceId !== voiceId) {
        updateSettings({ lastSelectedVoiceId: voiceId });
        changed = true;
      }
      if (modelId && configStore.settings.lastSelectedModelId !== modelId) {
        updateSettings({ lastSelectedModelId: modelId });
        changed = true;
      }
      if (language && configStore.settings.lastSelectedLanguage !== language) {
        updateSettings({ lastSelectedLanguage: language });
        changed = true;
      }
      if (changed) {
        saveConfig();
      }
    }
  });

  $effect(() => {
    if (isOpen && availableModels.length > 0 && availableLanguages.length > 0) {
      // Auto-update model and language if the current ones become invalid due to voice change
      if (!availableLanguages.find(l => l.id === language)) {
        language = availableLanguages[0]?.id;
      }
      if (!availableModels.find(m => m.model_id === modelId)) {
        modelId = availableModels[0].model_id;
      }
    }
  });

  function handleVoiceChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    voiceId = select.value;
    const voice = filteredVoices.find((v) => v.voice_id === voiceId);
    voiceName = voice?.name || "";
    // Clear audio if preview was playing
    if (currentAudio) {
      currentAudio.pause();
      currentAudio = null;
      isPlayingPreview = false;
    }
  }

  async function handleRefresh() {
    isRefreshing = true;
    try {
      await Promise.all([loadVoices(true), loadModels(true)]);
    } catch (e) {
      console.error("Failed to refresh", e);
    } finally {
      isRefreshing = false;
    }
  }

  async function handleGenerate() {
    if (!text.trim() || !voiceId || !modelId || isOverLimit) return;

    isGenerating = true;
    error = null;

    try {
      // ElevenLabs API only accepts base ISO 639-1 codes (e.g. "en", "ms").
      // Locale codes with regions ("en-US", "en-GB") cause a validation error.
      // Strip anything after a hyphen before sending.
      const baseLanguage = language ? language.split('-')[0] : undefined;

      const request: TTSGenerateRequest = {
        name: name.trim() || undefined,
        text: text.trim(),
        voiceId,
        voiceName,
        modelId,
        language: baseLanguage,
        stability: supportsSettings ? stability : undefined,
        similarityBoost: supportsSettings ? similarityBoost : undefined,
        speed: supportsSettings ? speed : undefined,
      };

      await generateSpeech(request);

      // Reset form
      text = "";
    } catch (e) {
      error = e instanceof Error ? e.message : "Generation failed";
    } finally {
      isGenerating = false;
    }
  }

  function handleClose() {
    closeTTSPanel();
    error = null;
    if (currentAudio) {
      currentAudio.pause();
      currentAudio = null;
      isPlayingPreview = false;
    }
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
    <div class="header-actions">
      <button class="action-btn" onclick={handleRefresh} title="Refresh Voices" disabled={isRefreshing}>
        <IconRefresh size={16} class={isRefreshing ? 'spinning' : ''} />
      </button>
      <button class="action-btn" onclick={handleClose} aria-label="Close panel">
        <IconX size={16} />
      </button>
    </div>
  </div>

  <div class="panel-body">
    <!-- Name Input -->
    <div class="field">
      <label class="field-label" for="tts-name"
        >{tr.tts.name || "Name (Optional)"}</label
      >
      <input
        id="tts-name"
        type="text"
        class="input"
        placeholder={tr.tts.namePlaceholder || "E.g., Opening Announcement"}
        bind:value={name}
      />
    </div>

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
          {charCount.toLocaleString()} / {maxCharacters.toLocaleString()}
          {tr.tts.charLimit}
        </span>
      </div>
    </div>

    <!-- Voice Selection -->
    <div class="field">
      <label class="field-label" for="tts-voice">{tr.tts.voice}</label>
      <div class="voice-select-row">
        <select
          id="tts-voice"
          class="select"
          value={voiceId}
          onchange={handleVoiceChange}
        >
          <option value="" disabled>{tr.tts.voicePlaceholder}</option>
          {#each filteredVoices as voice}
            <option value={voice.voice_id}>
              {voice.labels?.language ? getLanguageEmoji(voice.labels.language) + ' ' : ''}{voice.name}
            </option>
          {/each}
        </select>
        {#if selectedVoice?.preview_url}
          <button class="btn btn-ghost btn-icon preview-btn py-2" onclick={() => playPreview(selectedVoice?.preview_url)} title="Play Preview">
            {#if isPlayingPreview}
              <span><IconPlayerPause size={16} /></span>
            {:else}
              <span><IconPlayerPlay size={16} /></span>
            {/if}
          </button>
        {/if}
      </div>
    </div>

    <!-- Language Selection -->
    <div class="field">
      <label class="field-label" for="tts-language">{tr.tts.language}</label>
      <select id="tts-language" class="select" bind:value={language}>
        {#each availableLanguages as lang}
          <option value={lang.id}>{lang.label}</option>
        {/each}
      </select>
    </div>

    <!-- Model Selection -->
    <div class="field">
      <label class="field-label" for="tts-model">{tr.tts.model}</label>
      <select id="tts-model" class="select" bind:value={modelId}>
        <option value="" disabled>{tr.tts.modelPlaceholder}</option>
        {#each availableModels as model}
          <option value={model.model_id}>{model.name}</option>
        {/each}
      </select>
    </div>

    <!-- Settings Sliders (if model supports) -->
    {#if false}
      <!-- {#if supportsSettings} -->
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
            <label class="field-label" for="tts-stability"
              >{tr.tts.stability}</label
            >
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
            <label class="field-label" for="tts-similarity"
              >{tr.tts.similarity}</label
            >
            <span class="field-value"
              >{(similarityBoost * 100).toFixed(0)}%</span
            >
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
      disabled={!text.trim() ||
        !voiceId ||
        !modelId ||
        isOverLimit ||
        isGenerating}
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
    box-shadow: none;
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform 0.2s ease;
  }

  .panel.open {
    transform: translateX(0);
    box-shadow: var(--shadow-lg);
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

  .header-actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .action-btn:hover {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  .voice-select-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  
  .voice-select-row .select {
    flex: 1;
  }
  
  .preview-btn {
    padding: 0 12px;
    height: 36px;
    font-size: 14px;
    background: var(--color-surface-2);
  }
  
  .preview-btn:hover {
    background: var(--color-surface-3);
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
