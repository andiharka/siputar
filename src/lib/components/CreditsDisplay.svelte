<script lang="ts">
  import { t } from '$lib/i18n/index.svelte.js';
  import { ttsStore, formatCredits, getCreditsPercent } from '$lib/stores/tts.svelte.js';

  const tr = $derived(t());
  const subscription = $derived(ttsStore.subscription);
  const creditsText = $derived(formatCredits(subscription));
  const creditsPercent = $derived(getCreditsPercent(subscription));
</script>

<div class="credits-display">
  <span class="credits-label">{tr.tts.credits}:</span>
  <div class="credits-bar-container">
    <div class="credits-bar" style="width: {creditsPercent}%"></div>
  </div>
  <span class="credits-text">{creditsText}</span>
</div>

<style>
  .credits-display {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .credits-label {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .credits-bar-container {
    width: 80px;
    height: 6px;
    background: var(--color-surface-2);
    border-radius: 3px;
    overflow: hidden;
  }

  .credits-bar {
    height: 100%;
    background: var(--color-primary);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .credits-text {
    color: var(--color-text);
    font-weight: 500;
    min-width: 100px;
  }
</style>
