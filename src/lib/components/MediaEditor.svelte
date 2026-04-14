<script lang="ts">
  import {
    configStore,
    updateMedia,
    deleteMedia,
    addMedia,
  } from "$lib/stores/config.svelte.js";
  import { uiStore, showConfirm, closePanel } from "$lib/stores/ui.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { getFileName } from "$lib/utils/thumbnail.js";
  import { open } from "@tauri-apps/plugin-dialog";

  let { scheduleId, mediaId }: { scheduleId: string; mediaId: string } =
    $props();

  const tr = $derived(t());
  const schedule = $derived(
    configStore.schedules.find((s) => s.id === scheduleId),
  );
  const media = $derived(schedule?.media.find((m) => m.id === mediaId));

  function handleVolume(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    updateMedia(scheduleId, mediaId, { volume: val / 100 });
  }

  function handleLoop(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value) || 1;
    updateMedia(scheduleId, mediaId, { loopCount: Math.max(0, val) });
  }

  function handleDelete() {
    showConfirm(tr.actions.delete, tr.media.deleteConfirm, () => {
      deleteMedia(scheduleId, mediaId);
      closePanel();
    });
  }
</script>

{#if media}
  <div class="editor">
    <div class="file-path">
      <span class="field-label">File</span>
      <div class="path-display" title={media.path}>
        {getFileName(media.path)}
      </div>
    </div>

    <!-- Volume -->
    <div class="field">
      <label class="field-label" for="media-vol">
        {tr.media.volume}: <strong>{Math.round(media.volume * 100)}%</strong>
      </label>
      <input
        id="media-vol"
        type="range"
        min="0"
        max="100"
        step="1"
        value={media.volume * 100}
        oninput={handleVolume}
        class="range-input"
      />
    </div>

    <!-- Loop count -->
    <div class="field">
      <label class="field-label" for="media-loop">{tr.media.loopCount}</label>
      <div class="loop-col">
        <button
          class="spin-btn"
          type="button"
          onclick={() => updateMedia(scheduleId, mediaId, { loopCount: media.loopCount + 1 })}
          aria-label="Increase loop"
          tabindex="-1"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg>
        </button>
        <div class="loop-input-wrapper">
          <input
            id="media-loop"
            class="loop-input"
            type="text"
            inputmode="numeric"
            value={media.loopCount}
            onchange={handleLoop}
          />
          <span class="loop-x">x</span>
        </div>
        <button
          class="spin-btn"
          type="button"
          onclick={() => updateMedia(scheduleId, mediaId, { loopCount: Math.max(0, media.loopCount - 1) })}
          aria-label="Decrease loop"
          tabindex="-1"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
        </button>
      </div>
      <span class="loop-hint"
        >{media.loopCount === 0 ? tr.media.loopForever : ""}</span
      >
    </div>

    <div class="danger-zone">
      <button class="btn btn-danger" onclick={handleDelete} type="button">
        {tr.actions.delete}
      </button>
    </div>
  </div>
{/if}

<style>
  .editor {
    padding: 4px 0;
  }
  .section-title {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 20px;
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
  .file-path {
    margin-bottom: 20px;
  }
  .path-display {
    font-size: 13px;
    color: var(--color-text);
    background: var(--color-surface-3);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    word-break: break-all;
    border: 1px solid var(--color-border);
  }
  .range-input {
    width: 100%;
    accent-color: var(--color-primary);
  }
  .loop-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0px;
    width: fit-content;
  }
  .spin-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 22px;
    border: none;
    border-radius: var(--radius-md, 8px);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    padding: 0;
  }
  .spin-btn:hover {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
  }
  .spin-btn:active {
    background: color-mix(in srgb, var(--color-primary) 25%, transparent);
    transform: scale(0.92);
  }
  .loop-input-wrapper {
    position: relative;
    width: 64px;
  }
  .loop-input {
    width: 100%;
    height: 64px;
    text-align: center;
    font-size: 24px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    font-family: "SF Mono", "Cascadia Code", "Fira Code", "Consolas", monospace;
    letter-spacing: -0.5px;
    border: 1.5px solid var(--color-border);
    border-radius: var(--radius-md, 8px);
    background: var(--color-surface, #fff);
    color: var(--color-text);
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
    padding: 0 10px 0 0; /* space for 'x' */
    -moz-appearance: textfield;
    caret-color: var(--color-primary);
  }
  .loop-input::-webkit-outer-spin-button,
  .loop-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .loop-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 18%, transparent);
  }
  .loop-x {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-muted);
    pointer-events: none;
  }
  .loop-hint {
    display: block;
    margin-top: 8px;
    font-size: 12px;
    color: var(--color-primary);
    font-style: italic;
  }
  .danger-zone {
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
    margin-top: 8px;
  }
</style>
