<script lang="ts">
  import type { Schedule } from "$lib/types/index.js";
  import {
    addMedia,
    reorderMedia,
    deleteSchedule,
  } from "$lib/stores/config.svelte.js";
  import { openPanel, showConfirm } from "$lib/stores/ui.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { open } from "@tauri-apps/plugin-dialog";
  import MediaItemChip from "./MediaItemChip.svelte";
  import { formatDuration, calcTotalDuration } from "$lib/utils/duration.js";
  import {
    IconSettings,
    IconTrash,
    IconDisc,
    IconPlus,
    IconBellRinging,
  } from "@tabler/icons-svelte";

  let {
    schedule,
    onplay,
    highlight,
  }: {
    schedule: Schedule;
    onplay?: (scheduleId: string) => void;
    highlight?: boolean;
  } = $props();

  const tr = $derived(t());

  let durations = $state(new Map<string, number>());
  const totalDuration = $derived(calcTotalDuration(durations, schedule.media));

  const DAY_NUMS = [1, 2, 3, 4, 5, 6, 7];

  function handleMove(index: number, direction: -1 | 1) {
    const to = index + direction;
    if (to >= 0 && to < schedule.media.length) {
      reorderMedia(schedule.id, index, to);
    }
  }

  async function handleAddMedia() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Media",
            extensions: [
              "mp4",
              "mkv",
              "avi",
              "mov",
              "webm",
              "wmv",
              "mp3",
              "wav",
              "ogg",
              "flac",
              "m4a",
              "aac",
            ],
          },
        ],
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        addMedia(schedule.id, paths);
      }
    } catch {
      /* dialog not available in dev */
    }
  }

  function handleDeleteSchedule() {
    showConfirm(tr.actions.delete, tr.schedule.deleteConfirm, () =>
      deleteSchedule(schedule.id),
    );
  }

  function handleOpenSettings() {
    openPanel({ type: "schedule", scheduleId: schedule.id });
  }
</script>

<article class="card" class:disabled={!schedule.enabled} class:highlight>
  <div class="card-header">
    <div class="time-block">
      <IconBellRinging
        size={18}
        stroke={2}
        color={schedule.enabled ? "var(--color-primary)" : "#555"}
      />
      <span class="time">{schedule.time}</span>
      {#if !schedule.enabled}
        <span class="badge">Off</span>
      {/if}
    </div>

    <div class="days">
      {#each DAY_NUMS as day}
        <span
          class="badge"
          class:badge-day={schedule.activeDays.includes(day)}
          class:badge-day-inactive={!schedule.activeDays.includes(day)}
          >{tr.days[day as 1 | 2 | 3 | 4 | 5 | 6 | 7]}</span
        >
      {/each}
    </div>

    <div class="header-actions">
      {#if totalDuration > 0}
        <span class="duration">≈ {formatDuration(totalDuration)}</span>
      {/if}
      <button
        class="btn btn-ghost btn-icon"
        style="width: fit-content;"
        onclick={() => onplay?.(schedule.id)}
        title="Putar sekarang"><IconDisc size={15} />{tr.actions.play}</button
      >
      <button
        class="btn btn-ghost btn-icon"
        onclick={handleOpenSettings}
        title="Edit jadwal"><IconSettings size={15} /></button
      >
      <button
        class="btn btn-ghost btn-icon btn-danger"
        onclick={handleDeleteSchedule}
        title={tr.actions.delete}><IconTrash size={15} /></button
      >
    </div>
  </div>

  <div class="media-strip" role="list">
    {#each schedule.media as item, i (item.id)}
      <MediaItemChip
        media={item}
        scheduleId={schedule.id}
        index={i}
        isFirst={i === 0}
        isLast={i === schedule.media.length - 1}
        onmove={handleMove}
      />
    {/each}

    <button
      class="add-media-btn"
      onclick={handleAddMedia}
      title={tr.media.addMedia}
    >
      <IconPlus size={18} />
      <span class="add-label">{tr.media.addMedia}</span>
    </button>
  </div>
</article>

<style>
  .card {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
    padding: 16px;
    box-shadow: var(--shadow-sm);
    transition: var(--transition);
  }
  .card:hover {
    box-shadow: var(--shadow-md);
  }
  .card.disabled {
    opacity: 0.55;
  }
  .card.highlight {
    animation: card-highlight 1.4s ease-out forwards;
  }

  @keyframes card-highlight {
    0% {
      border-color: var(--color-primary);
      box-shadow: 0 0 0 3px
        color-mix(in srgb, var(--color-primary) 25%, transparent);
    }
    60% {
      border-color: var(--color-primary);
      box-shadow: 0 0 0 3px
        color-mix(in srgb, var(--color-primary) 25%, transparent);
    }
    100% {
      border-color: var(--color-border);
      box-shadow: var(--shadow-sm);
    }
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
    flex-wrap: wrap;
  }
  .time-block {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .time {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.5px;
    color: var(--color-text);
    font-variant-numeric: tabular-nums;
  }

  .days {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
  }
  .duration {
    font-size: 12px;
    color: var(--color-text-muted);
  }
  .btn-danger:hover {
    color: var(--color-danger) !important;
    background: color-mix(
      in srgb,
      var(--color-danger) 10%,
      transparent
    ) !important;
  }

  .media-strip {
    display: flex;
    gap: 10px;
    overflow-x: auto;
    /* padding-bottom: 4px; */
    align-items: flex-start;
    min-height: 60px;
  }
  .media-strip::-webkit-scrollbar {
    height: 4px;
  }

  .add-media-btn {
    flex-shrink: 0;
    height: 114px;
    width: 80px;
    /* aspect-ratio: 16/9; */
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-md);
    background: transparent;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    color: var(--color-text-muted);
    font-size: 20px;
    transition: var(--transition);
    padding: 2rem;
  }
  .add-media-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }
  .add-label {
    font-size: 10px;
    font-weight: 500;
  }
</style>
