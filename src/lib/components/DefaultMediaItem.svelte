<script lang="ts">
  import { onMount } from "svelte";
  import { getFileName, getMediaDuration } from "$lib/utils/thumbnail.js";
  import { formatDuration } from "$lib/utils/duration.js";
  import {
    IconPlayerPlay,
    IconPlayerStop,
    IconPlus,
    IconInfoCircle,
  } from "@tabler/icons-svelte";

  let { path, onSelect }: { path: string; onSelect: (path: string) => void } =
    $props();

  let duration = $state(0);
  let audioEl = $state<HTMLAudioElement | null>(null);
  let isPlaying = $state(false);

  const ATTRIBUTIONS: Record<string, string> = {
    "Blank 1s.mp3":
      "This audio is intentionally blank to create spacing between media items",
    "Airport Start.mp3":
      "Jingle 5 by ElevatorFan2020 -- https://freesound.org/s/749217/ -- License: Creative Commons 0",
    "Airport End.mp3":
      "Jingle 5 by ElevatorFan2020 -- https://freesound.org/s/749217/ -- License: Creative Commons 0",
    "Alarm.mp3":
      "Analog Alarm Clock by bone666138 -- https://freesound.org/s/198841/ -- License: Attribution 4.0",
    "Bengbong.mp3":
      "Bingbong.wav by Benboncan -- https://freesound.org/s/76925/ -- License: Attribution 4.0",
    "Tritone Chime.mp3":
      "A320 tritone chime.wav by nachosose -- https://freesound.org/s/244583/ -- License: Creative Commons 0",
    "Do Re Mi Fa So.mp3":
      "marimba do re mi fa so.WAV by pogmothoin -- https://freesound.org/s/401722/ -- License: Attribution 4.0",
  };
  const fileName = $derived(getFileName(path));
  const attribution = $derived(ATTRIBUTIONS[fileName]);

  onMount(() => {
    // use path as mediaId since it's unique
    getMediaDuration(path, path, "audio").then((d) => {
      duration = d;
    });
  });

  function togglePlay(e: Event) {
    e.stopPropagation();
    if (!audioEl) {
      audioEl = new Audio(path);
      audioEl.onended = () => {
        isPlaying = false;
      };
      audioEl.onpause = () => {
        isPlaying = false;
      };
      audioEl.onplay = () => {
        isPlaying = true;
      };
    }

    if (isPlaying) {
      audioEl.pause();
      audioEl.currentTime = 0;
    } else {
      audioEl.play().catch(console.error);
    }
  }
</script>

<div class="media-item">
  <button
    class="play-btn"
    onclick={togglePlay}
    title={isPlaying ? "Stop" : "Play"}
  >
    {#if isPlaying}
      <IconPlayerStop size={18} />
    {:else}
      <IconPlayerPlay size={18} />
    {/if}
  </button>

  <div class="info">
    <span class="name">{fileName}</span>
    <div class="meta">
      {#if duration > 0}
        <span class="duration">{formatDuration(duration)}</span>
      {/if}
      {#if attribution}
        <div class="attr-btn tt">
          <IconInfoCircle size={18} style="opacity: .5;" />
          <span class="tt__bubble attr-bubble">{attribution}</span>
        </div>
      {/if}
    </div>
  </div>

  <button
    class="add-btn"
    onclick={() => onSelect(path)}
    title="Add to schedule"
  >
    <IconPlus size={18} />
  </button>
</div>

<style>
  .media-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: var(--transition);
  }
  .media-item:hover {
    border-color: var(--color-primary);
  }

  .play-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    transition: var(--transition);
  }
  .play-btn:hover {
    background: var(--color-primary);
    color: white;
    transform: scale(1.05);
  }

  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .name {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .duration {
    font-size: 12px;
    color: var(--color-text-muted);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .attr-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    cursor: help;
    transition: var(--transition);
  }
  .attr-btn:hover {
    color: var(--color-primary);
  }
  .attr-bubble {
    white-space: normal;
    width: max-content;
    max-width: 220px;
    text-align: center;
    line-height: 1.4;
  }

  .add-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    color: white;
    border: none;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: var(--transition);
  }
  .add-btn:hover {
    filter: brightness(1.1);
  }
</style>
