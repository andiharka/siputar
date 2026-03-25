<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import ScheduleCard from "./ScheduleCard.svelte";
  import { compareTime } from "$lib/utils/time.js";
  import { IconTimelineEventExclamation } from "@tabler/icons-svelte";

  let {
    onplay,
    newScheduleId = null,
  }: {
    onplay?: (scheduleId: string) => void;
    newScheduleId?: string | null;
  } = $props();

  const tr = $derived(t());
  const sorted = $derived(
    [...configStore.schedules].sort((a, b) => compareTime(a.time, b.time)),
  );
</script>

<div class="list">
  {#if sorted.length === 0}
    <div class="empty">
      <div class="empty-icon">
        <IconTimelineEventExclamation size={96} stroke={0.625} />
      </div>
      <p>{tr.schedule.noSchedules}</p>
    </div>
  {:else}
    {#each sorted as schedule (schedule.id)}
      <div id="schedule-{schedule.id}">
        <ScheduleCard
          {schedule}
          {onplay}
          highlight={schedule.id === newScheduleId}
        />
      </div>
    {/each}
  {/if}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
    max-width: 960px;
    margin: 0 auto;
    width: 100%;
  }

  .empty {
    text-align: center;
    padding: 60px 20px;
    color: var(--color-text-muted);
  }
  .empty-icon {
    margin-bottom: 16px;
  }
  .empty p {
    font-size: 14px;
  }
</style>
