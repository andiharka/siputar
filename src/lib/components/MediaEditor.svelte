<script lang="ts">
  import { configStore, updateMedia, deleteMedia, addMedia } from '$lib/stores/config.svelte.js';
  import { uiStore, showConfirm, closePanel } from '$lib/stores/ui.svelte.js';
  import { t } from '$lib/i18n/index.svelte.js';
  import { getFileName } from '$lib/utils/thumbnail.js';
  import { open } from '@tauri-apps/plugin-dialog';

  let { scheduleId, mediaId }: { scheduleId: string; mediaId: string } = $props();

  const tr = $derived(t());
  const schedule = $derived(configStore.schedules.find(s => s.id === scheduleId));
  const media = $derived(schedule?.media.find(m => m.id === mediaId));

  function handleVolume(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    updateMedia(scheduleId, mediaId, { volume: val / 100 });
  }

  function handleLoop(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value) || 1;
    updateMedia(scheduleId, mediaId, { loopCount: Math.max(0, val) });
  }

  function handleDelete() {
    showConfirm(
      tr.actions.delete,
      tr.media.deleteConfirm,
      () => {
        deleteMedia(scheduleId, mediaId);
        closePanel();
      }
    );
  }
</script>

{#if media}
  <div class="editor">
    <h3 class="section-title">{tr.media.title}</h3>

    <div class="file-path">
      <span class="field-label">File</span>
      <div class="path-display" title={media.path}>{getFileName(media.path)}</div>
    </div>

    <!-- Volume -->
    <div class="field">
      <label class="field-label" for="media-vol">
        {tr.media.volume}: <strong>{Math.round(media.volume * 100)}%</strong>
      </label>
      <input
        id="media-vol"
        type="range" min="0" max="100" step="1"
        value={media.volume * 100}
        oninput={handleVolume}
        class="range-input"
      />
    </div>

    <!-- Loop count -->
    <div class="field">
      <label class="field-label" for="media-loop">{tr.media.loopCount}</label>
      <div class="loop-row">
        <input
          id="media-loop"
          class="input"
          type="number" min="0" step="1"
          value={media.loopCount}
          oninput={handleLoop}
          style="width: 80px;"
        />
        <span class="loop-hint">{media.loopCount === 0 ? tr.media.loopForever : ''}</span>
      </div>
    </div>

    <div class="danger-zone">
      <button class="btn btn-danger" onclick={handleDelete} type="button">
        {tr.actions.delete}
      </button>
    </div>
  </div>
{/if}

<style>
  .editor { padding: 4px 0; }
  .section-title { font-size: 15px; font-weight: 600; margin-bottom: 20px; }
  .field { margin-bottom: 20px; }
  .field-label { display: block; font-size: 12px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
  .file-path { margin-bottom: 20px; }
  .path-display {
    font-size: 13px; color: var(--color-text);
    background: var(--color-surface-2); border-radius: var(--radius-sm);
    padding: 6px 10px; word-break: break-all;
    border: 1px solid var(--color-border);
  }
  .range-input { width: 100%; accent-color: var(--color-primary); }
  .loop-row { display: flex; align-items: center; gap: 10px; }
  .loop-hint { font-size: 12px; color: var(--color-primary); font-style: italic; }
  .danger-zone { padding-top: 12px; border-top: 1px solid var(--color-border); margin-top: 8px; }
</style>
