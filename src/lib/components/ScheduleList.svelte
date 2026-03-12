<script lang="ts">
  import { tick } from "svelte";
  import { configStore, addSchedule } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import ScheduleCard from "./ScheduleCard.svelte";
  import { compareTime } from "$lib/utils/time.js";
  import { IconCalendarPlus } from "@tabler/icons-svelte";

  let { onplay }: { onplay?: (scheduleId: string) => void } = $props();

  const tr = $derived(t());
  const sorted = $derived(
    [...configStore.schedules].sort((a, b) => compareTime(a.time, b.time)),
  );

  let newScheduleId = $state<string | null>(null);

  async function handleAddSchedule() {
    const schedule = addSchedule();
    newScheduleId = schedule.id;
    await tick();
    // Scroll the new card into view
    const el = document.getElementById(`schedule-${schedule.id}`);
    el?.scrollIntoView({ behavior: "smooth", block: "center" });
    // Clear highlight after animation completes
    setTimeout(() => {
      newScheduleId = null;
    }, 1600);
  }
</script>

<div class="list">
  {#if sorted.length === 0}
    <div class="empty">
      <div class="empty-icon">📅</div>
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

  <button class="add-btn" onclick={handleAddSchedule}>
    <IconCalendarPlus size={18} />
    {tr.schedule.addSchedule}
  </button>
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
    font-size: 48px;
    margin-bottom: 16px;
  }
  .empty p {
    font-size: 14px;
  }

  .add-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 2rem;
    border-radius: var(--radius-lg);
    border: 2px dashed var(--color-border);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
    justify-content: center;
  }
  .add-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }
</style>
