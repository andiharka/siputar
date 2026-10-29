<script lang="ts">
  import { onMount } from "svelte";
  import { IconCopy, IconX } from "@tabler/icons-svelte";
  import { duplicateSchedule } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { ALL_DAYS, type Schedule } from "$lib/types/index.js";
  import { isValidTime } from "$lib/utils/time.js";

  let {
    schedule,
    onclose,
    oncreated,
  }: {
    schedule: Schedule;
    onclose: () => void;
    oncreated: (id: string) => void;
  } = $props();

  const tr = $derived(t());
  let dialog: HTMLDialogElement;
  let nameInput: HTMLInputElement;
  let name = $state("");
  let time = $state("");
  let activeDays = $state<number[]>([]);
  const normalizedTime = $derived(time.length === 5 ? `${time}:00` : time);
  const valid = $derived(name.trim().length > 0 && isValidTime(normalizedTime));

  onMount(() => {
    name = `${schedule.name?.trim() || tr.schedule.title} copy`;
    time = schedule.time;
    activeDays = [...schedule.activeDays];
    dialog.showModal();
    nameInput.select();
  });

  function toggleDay(day: number) {
    activeDays = activeDays.includes(day)
      ? activeDays.filter(d => d !== day)
      : [...activeDays, day].sort((a, b) => a - b);
  }

  function handleClose() {
    dialog.close();
    onclose();
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    const copy = duplicateSchedule(schedule.id, {
      name: name.trim(),
      time: normalizedTime,
      activeDays,
    });
    handleClose();
    if (copy) oncreated(copy.id);
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="duplicate-schedule-title"
  aria-describedby="duplicate-schedule-description"
  oncancel={(event) => { event.preventDefault(); handleClose(); }}
>
  <form onsubmit={handleSubmit}>
    <div class="dialog-header">
      <h2 id="duplicate-schedule-title">{tr.schedule.duplicate}</h2>
      <button
        type="button"
        class="btn btn-ghost btn-icon"
        onclick={handleClose}
        aria-label={tr.actions.close}
      ><IconX size={16} /></button>
    </div>
    <p id="duplicate-schedule-description">{tr.schedule.duplicateDescription}</p>

    <label class="field">
      <span>{tr.schedule.name}</span>
      <input
        bind:this={nameInput}
        bind:value={name}
        class="input"
        type="text"
        placeholder={tr.schedule.namePlaceholder}
        required
      />
    </label>

    <label class="field">
      <span>{tr.schedule.time}</span>
      <input class="input" type="time" step="1" bind:value={time} required />
    </label>

    <fieldset>
      <legend>{tr.schedule.activeDays}</legend>
      <div class="days">
        {#each ALL_DAYS as day}
          <button
            type="button"
            class="day"
            class:selected={activeDays.includes(day)}
            aria-pressed={activeDays.includes(day)}
            aria-label={tr.days.fullName[day as keyof typeof tr.days.fullName]}
            onclick={() => toggleDay(day)}
          >{tr.days[day as keyof typeof tr.days.fullName]}</button>
        {/each}
      </div>
    </fieldset>

    <div class="actions">
      <button type="button" class="btn btn-ghost" onclick={handleClose}>{tr.actions.cancel}</button>
      <button type="submit" class="btn btn-primary" disabled={!valid}>
        <IconCopy size={16} />{tr.schedule.duplicate}
      </button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    margin: auto;
    width: 440px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 32px);
    padding: 24px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: var(--shadow-lg);
  }
  dialog::backdrop {
    background: var(--color-overlay);
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    font-size: 17px;
    font-weight: 600;
  }
  p {
    margin-top: -12px;
    color: var(--color-text-muted);
    font-size: 13px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
  }
  .input {
    min-height: 38px;
    color-scheme: light dark;
  }
  fieldset {
    min-width: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 500;
  }
  .days {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .day {
    flex: 1;
    min-width: 36px;
    padding: 8px 2px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-3);
    color: var(--color-text-muted);
    font-size: 12px;
    cursor: pointer;
  }
  .day.selected {
    border-color: var(--color-primary);
    background: var(--color-surface-2);
    color: var(--color-primary);
    font-weight: 600;
  }
  .day:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 8px;
    padding-top: 4px;
  }
</style>
