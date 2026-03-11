<script lang="ts">
  import type { Schedule } from '$lib/types/index.js';
  import { configStore, updateSchedule, addNotification, removeNotification, updateNotification } from '$lib/stores/config.svelte.js';
  import { t } from '$lib/i18n/index.svelte.js';
  import { isValidTime } from '$lib/utils/time.js';

  let { scheduleId }: { scheduleId: string } = $props();

  const tr = $derived(t());
  const schedule = $derived(configStore.schedules.find(s => s.id === scheduleId));

  const DAY_LABELS = $derived([
    [1, tr.days[1]], [2, tr.days[2]], [3, tr.days[3]], [4, tr.days[4]],
    [5, tr.days[5]], [6, tr.days[6]], [7, tr.days[7]],
  ] as [number, string][]);

  function toggleDay(day: number) {
    if (!schedule) return;
    const current = schedule.activeDays;
    const updated = current.includes(day)
      ? current.filter(d => d !== day)
      : [...current, day].sort((a, b) => a - b);
    updateSchedule(scheduleId, { activeDays: updated });
  }

  function handleTimeInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    if (isValidTime(val)) updateSchedule(scheduleId, { time: val });
  }
</script>

{#if schedule}
  <div class="editor">
    <h3 class="section-title">{tr.schedule.title}</h3>

    <!-- Enabled toggle -->
    <div class="field field-row">
      <span class="field-label">{tr.schedule.enabled}</span>
      <label class="toggle">
        <input
          type="checkbox"
          checked={schedule.enabled}
          onchange={(e) => updateSchedule(scheduleId, { enabled: (e.target as HTMLInputElement).checked })}
        />
        <span class="toggle-slider"></span>
      </label>
    </div>

    <!-- Time -->
    <div class="field">
      <label class="field-label" for="sched-time">{tr.schedule.time}</label>
      <input
        id="sched-time"
        class="input"
        type="text"
        placeholder="HH:MM:SS"
        value={schedule.time}
        oninput={handleTimeInput}
        pattern="\d{2}:\d{2}:\d{2}"
      />
    </div>

    <!-- Active days -->
    <div class="field">
      <span class="field-label">{tr.schedule.activeDays}</span>
      <div class="day-grid">
        {#each DAY_LABELS as [day, label]}
          <button
            class="day-btn"
            class:day-active={schedule.activeDays.includes(day)}
            onclick={() => toggleDay(day)}
            type="button"
          >{label}</button>
        {/each}
      </div>
    </div>

    <!-- Notifications -->
    <div class="field">
      <span class="field-label">{tr.schedule.notifications}</span>
      {#each schedule.notifications as notif, i}
        <div class="notif-row">
          <input
            class="input input-sm notif-input"
            type="number"
            min="1"
            max="60"
            value={notif.offsetMinutes}
            oninput={(e) => updateNotification(scheduleId, i, { offsetMinutes: parseInt((e.target as HTMLInputElement).value) || 1 })}
          />
          <span class="notif-label">{tr.schedule.minutesBefore}</span>
          <button
            class="btn btn-ghost btn-icon"
            onclick={() => removeNotification(scheduleId, i)}
            type="button"
            aria-label="Remove"
          >✕</button>
        </div>
      {/each}
      <button
        class="btn btn-ghost"
        style="margin-top: 6px; font-size: 12px;"
        onclick={() => addNotification(scheduleId, { offsetMinutes: 1 })}
        type="button"
      >+ {tr.schedule.addNotification}</button>
    </div>
  </div>
{/if}

<style>
  .editor { padding: 4px 0; }
  .section-title { font-size: 15px; font-weight: 600; margin-bottom: 20px; }
  .field { margin-bottom: 20px; }
  .field-label { display: block; font-size: 12px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
  .field-row { display: flex; align-items: center; justify-content: space-between; }
  .field-row .field-label { margin-bottom: 0; }
  .day-grid { display: flex; gap: 6px; flex-wrap: wrap; }
  .day-btn {
    padding: 4px 10px; border-radius: 100px; font-size: 12px; font-weight: 600;
    border: 1px solid var(--color-border); background: transparent; cursor: pointer;
    color: var(--color-text-muted); transition: var(--transition);
  }
  .day-btn.day-active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
  .notif-row { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
  .notif-input { width: 60px; flex-shrink: 0; }
  .notif-label { font-size: 12px; color: var(--color-text-muted); flex: 1; }
</style>
