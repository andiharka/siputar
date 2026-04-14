<script lang="ts">
  import { onMount } from "svelte";
  import type { MediaItem } from "$lib/types/index.js";
  import {
    getMediaType,
    getFileName,
    getVideoThumbnail,
    getMediaDuration,
  } from "$lib/utils/thumbnail.js";
  import { openPanel } from "$lib/stores/ui.svelte.js";
  import { validationStore } from "$lib/stores/config.svelte.js";
  import {
    IconChevronLeft,
    IconChevronRight,
    IconPencil,
  } from "@tabler/icons-svelte";
  import { formatDuration } from "$lib/utils/duration.js";

  let {
    media,
    scheduleId,
    index,
    isFirst,
    isLast,
    onmove,
  }: {
    media: MediaItem;
    scheduleId: string;
    index: number;
    isFirst: boolean;
    isLast: boolean;
    onmove?: (index: number, direction: -1 | 1) => void;
  } = $props();

  const type = $derived(getMediaType(media.path));
  const fileName = $derived(getFileName(media.path));
  const isMissing = $derived(validationStore.isMissing(media.path));

  let thumbnail = $state<string | null>(null);
  let duration = $state(0);

  onMount(() => {
    if (!isMissing) {
      if (type === "video") {
        getVideoThumbnail(media.id, media.path).then((t) => {
          thumbnail = t;
        });
      }
      getMediaDuration(media.id, media.path, type).then((d) => {
        duration = d;
      });
    }
  });

  function handleClick() {
    openPanel({ type: "media", scheduleId, mediaId: media.id });
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="chip"
  class:missing={isMissing}
  onclick={handleClick}
  role="button"
  tabindex="0"
>
  <div class="thumb">
    {#if isMissing}
      <div class="thumb-placeholder missing-icon">!</div>
    {:else if type === "video"}
      {#if thumbnail}
        <img src={thumbnail} alt={fileName} class="thumb-img" />
      {:else}
        <div class="thumb-placeholder video">▶</div>
      {/if}
    {:else}
      <div class="thumb-placeholder audio">♪</div>
    {/if}

    <!-- Move buttons overlay (visible on hover) -->
    {#if onmove}
      <div class="move-overlay">
        <button
          class="move-btn"
          class:invisible={isFirst}
          onclick={(e) => {
            e.stopPropagation();
            onmove?.(index, -1);
          }}
          title="Pindah kiri"
          tabindex="-1"><IconChevronLeft size={14} /></button
        >
        <button
          class="move-btn"
          class:invisible={isLast}
          onclick={(e) => {
            e.stopPropagation();
            onmove?.(index, 1);
          }}
          title="Pindah kanan"
          tabindex="-1"><IconChevronRight size={14} /></button
        >
      </div>
    {/if}

    <!-- Edit hint (visible on hover) -->
    <div class="edit-hint">
      <IconPencil size={24} stroke={1.5} />
    </div>
  </div>
  <div class="info">
    <span class="name" title={fileName}>{fileName}</span>
    <div class="meta-row">
      {#if duration > 0}
        <span class="duration-badge">{formatDuration(duration)}</span>
      {/if}
      {#if isMissing}
        <span class="missing-badge">!</span>
      {:else if media.loopCount === 0}
        <span class="loop-badge">∞</span>
      {:else if media.loopCount > 1}
        <span class="loop-badge">{media.loopCount}x</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .chip {
    flex-shrink: 0;
    width: 120px;
    cursor: pointer;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface-3);
    border: 1px solid var(--color-border);
    transition: var(--transition);
    user-select: none;
    position: relative;
  }
  .chip:hover {
    border-color: var(--color-primary);
  }
  .chip.missing {
    border-color: var(--color-danger);
    opacity: 0.8;
  }

  .thumb {
    width: 100%;
    aspect-ratio: 16/9;
    overflow: hidden;
    background: var(--color-border);
    position: relative;
  }
  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .thumb-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 22px;
    color: var(--color-text-muted);
  }
  .thumb-placeholder.audio {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
  }
  .thumb-placeholder.missing-icon {
    background: color-mix(in srgb, var(--color-danger) 15%, transparent);
    color: var(--color-danger);
    font-weight: 700;
  }

  /* Move overlay — shown on chip hover */
  .move-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 3px;
    opacity: 0;
    transition: opacity 0.15s;
    background: rgba(0, 0, 0, 0.35);
  }
  .chip:hover .move-overlay {
    opacity: 1;
  }

  /* Edit hint — pencil icon on hover */
  .edit-hint {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.9);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-primary);
    opacity: 0;
    transition: 0.15s;
  }
  .chip:hover .edit-hint {
    opacity: 1;
  }
  .edit-hint:hover {
    background: white;
    scale: 1.1;
  }

  .move-btn {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.9);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #222;
    padding: 0;
    transition: 0.15s;
    flex-shrink: 0;
  }
  .move-btn:hover {
    background: white;
    transform: scale(1.1);
  }
  .move-btn.invisible {
    opacity: 0;
    pointer-events: none;
  }

  .info {
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .name {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    flex-wrap: wrap;
  }
  .duration-badge {
    font-size: 10px;
    color: var(--color-text-muted);
  }
  .loop-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--color-primary);
    background: color-mix(in srgb, var(--color-primary) 12%, transparent);
    border-radius: 4px;
    padding: 1px 5px;
    align-self: flex-start;
  }
  .missing-badge {
    font-size: 10px;
    font-weight: 700;
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 12%, transparent);
    border-radius: 4px;
    padding: 1px 5px;
    align-self: flex-start;
  }
</style>
