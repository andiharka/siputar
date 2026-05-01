<script lang="ts">
  import { t } from "$lib/i18n/index.svelte.js";
  import { addMedia } from "$lib/stores/config.svelte.js";
  import { closePanel } from "$lib/stores/ui.svelte.js";
  import DefaultMediaItem from "./DefaultMediaItem.svelte";

  let { scheduleId }: { scheduleId: string } = $props();

  const tr = $derived(t());

  const DEFAULT_MEDIA_FILES = [
    "/media/Blank 1s.mp3",
    "/media/Airport Start.mp3",
    "/media/Airport End.mp3",
    "/media/Alarm.mp3",
    "/media/Bengbong.mp3",
    "/media/Do Re Mi Fa So.mp3",
    "/media/Tritone Chime.mp3",
  ];

  function handleSelect(path: string) {
    addMedia(scheduleId, [path]);
    closePanel();
  }
</script>

<div class="picker">
  <p class="description">{tr.media.defaultMediaDescription}</p>
  <div class="media-list">
    {#each DEFAULT_MEDIA_FILES as path}
      <DefaultMediaItem {path} onSelect={handleSelect} />
    {/each}
  </div>
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .description {
    font-size: 14px;
    color: var(--color-text-muted);
    margin: 0;
  }
  .media-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
