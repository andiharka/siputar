<script lang="ts">
  import type { Schedule } from "$lib/types/index.js";
  import {
    configStore,
    updateSchedule,
    addNotification,
    removeNotification,
    updateNotification,
  } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { isValidTime } from "$lib/utils/time.js";

  let { scheduleId }: { scheduleId: string } = $props();

  const tr = $derived(t());
  const schedule = $derived(
    configStore.schedules.find((s) => s.id === scheduleId),
  );

  const DAY_LABELS = $derived([
    [1, tr.days[1]],
    [2, tr.days[2]],
    [3, tr.days[3]],
    [4, tr.days[4]],
    [5, tr.days[5]],
    [6, tr.days[6]],
    [7, tr.days[7]],
  ] as [number, string][]);

  // Parse time parts from schedule
  const hours = $derived(schedule ? parseInt(schedule.time.slice(0, 2)) : 0);
  const minutes = $derived(schedule ? parseInt(schedule.time.slice(3, 5)) : 0);
  const seconds = $derived(schedule ? parseInt(schedule.time.slice(6, 8)) : 0);

  function pad(n: number): string {
    return String(n).padStart(2, "0");
  }

  function setTime(h: number, m: number, s: number) {
    // Wrap around
    if (s < 0) {
      s = 59;
      m--;
    }
    if (s > 59) {
      s = 0;
      m++;
    }
    if (m < 0) {
      m = 59;
      h--;
    }
    if (m > 59) {
      m = 0;
      h++;
    }
    if (h < 0) h = 23;
    if (h > 23) h = 0;
    const time = `${pad(h)}:${pad(m)}:${pad(s)}`;
    if (isValidTime(time)) updateSchedule(scheduleId, { time });
  }

  function incrementHour() {
    setTime(hours + 1, minutes, seconds);
  }
  function decrementHour() {
    setTime(hours - 1, minutes, seconds);
  }
  function incrementMinute() {
    setTime(hours, minutes + 1, seconds);
  }
  function decrementMinute() {
    setTime(hours, minutes - 1, seconds);
  }
  function incrementSecond() {
    setTime(hours, minutes, seconds + 1);
  }
  function decrementSecond() {
    setTime(hours, minutes, seconds - 1);
  }

  function handleUnitInput(unit: "h" | "m" | "s", e: Event) {
    const input = e.target as HTMLInputElement;
    let val = parseInt(input.value);
    if (isNaN(val)) return;
    const max = unit === "h" ? 23 : 59;
    val = Math.max(0, Math.min(max, val));
    input.value = pad(val);
    if (unit === "h") setTime(val, minutes, seconds);
    else if (unit === "m") setTime(hours, val, seconds);
    else setTime(hours, minutes, val);
  }

  function handleUnitFocus(e: Event) {
    (e.target as HTMLInputElement).select();
  }

  function handleUnitKeydown(unit: "h" | "m" | "s", e: KeyboardEvent) {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (unit === "h") incrementHour();
      else if (unit === "m") incrementMinute();
      else incrementSecond();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (unit === "h") decrementHour();
      else if (unit === "m") decrementMinute();
      else decrementSecond();
    }
  }

  function toggleDay(day: number) {
    if (!schedule) return;
    const current = schedule.activeDays;
    const updated = current.includes(day)
      ? current.filter((d) => d !== day)
      : [...current, day].sort((a, b) => a - b);
    updateSchedule(scheduleId, { activeDays: updated });
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
          onchange={(e) =>
            updateSchedule(scheduleId, {
              enabled: (e.target as HTMLInputElement).checked,
            })}
        />
        <span class="toggle-slider"></span>
      </label>
    </div>

    <!-- Time picker -->
    <div class="field">
      <span class="field-label">{tr.schedule.time}</span>
      <div class="time-picker">
        <!-- Hours -->
        <div class="time-col">
          <button
            class="spin-btn"
            type="button"
            onclick={incrementHour}
            aria-label="Increase hour"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg
            >
          </button>
          <input
            class="time-input"
            type="text"
            inputmode="numeric"
            maxlength={2}
            value={pad(hours)}
            onchange={(e) => handleUnitInput("h", e)}
            onfocus={handleUnitFocus}
            onkeydown={(e) => handleUnitKeydown("h", e)}
            aria-label="Hours"
          />
          <button
            class="spin-btn"
            type="button"
            onclick={decrementHour}
            aria-label="Decrease hour"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg
            >
          </button>
          <span class="time-unit-label">{tr.schedule.hours}</span>
        </div>

        <span class="time-sep">:</span>

        <!-- Minutes -->
        <div class="time-col">
          <button
            class="spin-btn"
            type="button"
            onclick={incrementMinute}
            aria-label="Increase minute"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg
            >
          </button>
          <input
            class="time-input"
            type="text"
            inputmode="numeric"
            maxlength={2}
            value={pad(minutes)}
            onchange={(e) => handleUnitInput("m", e)}
            onfocus={handleUnitFocus}
            onkeydown={(e) => handleUnitKeydown("m", e)}
            aria-label="Minutes"
          />
          <button
            class="spin-btn"
            type="button"
            onclick={decrementMinute}
            aria-label="Decrease minute"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg
            >
          </button>
          <span class="time-unit-label">{tr.schedule.minutes}</span>
        </div>

        <span class="time-sep">:</span>

        <!-- Seconds -->
        <div class="time-col">
          <button
            class="spin-btn"
            type="button"
            onclick={incrementSecond}
            aria-label="Increase second"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="18 15 12 9 6 15" /></svg
            >
          </button>
          <input
            class="time-input"
            type="text"
            inputmode="numeric"
            maxlength={2}
            value={pad(seconds)}
            onchange={(e) => handleUnitInput("s", e)}
            onfocus={handleUnitFocus}
            onkeydown={(e) => handleUnitKeydown("s", e)}
            aria-label="Seconds"
          />
          <button
            class="spin-btn"
            type="button"
            onclick={decrementSecond}
            aria-label="Decrease second"
            tabindex="-1"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg
            >
          </button>
          <span class="time-unit-label">{tr.schedule.seconds}</span>
        </div>
      </div>
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
            type="button">{label}</button
          >
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
            oninput={(e) =>
              updateNotification(scheduleId, i, {
                offsetMinutes:
                  parseInt((e.target as HTMLInputElement).value) || 1,
              })}
          />
          <span class="notif-label">{tr.schedule.minutesBefore}</span>
          <button
            class="btn btn-ghost btn-icon"
            onclick={() => removeNotification(scheduleId, i)}
            type="button"
            aria-label="Remove">✕</button
          >
        </div>
      {/each}
      <button
        class="btn btn-ghost"
        style="margin-top: 6px; font-size: 12px;"
        onclick={() => addNotification(scheduleId, { offsetMinutes: 1 })}
        type="button">+ {tr.schedule.addNotification}</button
      >
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
  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .field-row .field-label {
    margin-bottom: 0;
  }

  /* ── Time picker ── */
  .time-picker {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .time-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
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
    transition:
      background 0.15s,
      color 0.15s;
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
  .time-input {
    width: 48px;
    height: 48px;
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
    transition:
      border-color 0.2s,
      box-shadow 0.2s;
    padding: 0;
    -moz-appearance: textfield;
    caret-color: var(--color-primary);
  }
  .time-input::-webkit-outer-spin-button,
  .time-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .time-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px
      color-mix(in srgb, var(--color-primary) 18%, transparent);
  }
  .time-unit-label {
    font-size: 9px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.6;
    margin-top: 1px;
  }
  .time-sep {
    font-size: 28px;
    font-weight: 700;
    color: var(--color-text-muted);
    line-height: 1;
    padding: 0 2px;
    margin-bottom: 18px; /* offset to align with inputs, not labels */
    user-select: none;
  }

  /* ── Days ── */
  .day-grid {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .day-btn {
    padding: 4px 10px;
    border-radius: 100px;
    font-size: 12px;
    font-weight: 600;
    border: 1px solid var(--color-border);
    background: transparent;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: var(--transition);
  }
  .day-btn.day-active {
    background: var(--color-primary);
    color: #fff;
    border-color: var(--color-primary);
  }

  /* ── Notifications ── */
  .notif-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .notif-input {
    width: 60px;
    flex-shrink: 0;
  }
  .notif-label {
    font-size: 12px;
    color: var(--color-text-muted);
    flex: 1;
  }
</style>
