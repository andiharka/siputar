<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    configStore,
    saveConfig,
    revertConfig,
    addSchedule,
  } from "$lib/stores/config.svelte.js";
  import {
    playbackStore,
    setSchedulerStatus,
  } from "$lib/stores/playback.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { startScheduledPlayback } from "$lib/scheduledPlayback.js";
  import ScheduleList from "$lib/components/ScheduleList.svelte";
  import ConfigPanel from "$lib/components/ConfigPanel.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import {
    IconPlayerPause,
    IconPlayerPlay,
    IconPlus,
  } from "@tabler/icons-svelte";

  const tr = $derived(t());
  const isDirty = $derived(configStore.isDirty);
  const schedulerStatus = $derived(playbackStore.schedulerStatus);
  let newScheduleId = $state<string | null>(null);

  async function handleToggleScheduler() {
    if (schedulerStatus === "active") {
      await invoke("pause_all").catch(() => {});
      setSchedulerStatus("paused");
    } else {
      await invoke("resume_all").catch(() => {});
      setSchedulerStatus("active");
    }
  }

  async function handleAddSchedule() {
    const { tick } = await import("svelte");
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

<svelte:head><title>{tr.nav.schedules} - {tr.app.name}</title></svelte:head>

<div class="page-header">
  <div class="header-left">
    <button
      class="btn badge tt"
      onclick={handleToggleScheduler}
      class:badge-active={schedulerStatus === "active"}
      class:badge-paused={schedulerStatus === "paused"}
    >
      {#if schedulerStatus === "active"}
        <IconPlayerPlay size={16} />
        {tr.status.active}
        <span class="tooltip">- {tr.schedule.statusEnabled}</span>
      {:else}
        <IconPlayerPause size={16} />
        {tr.status.paused}
        <span class="tooltip">- {tr.schedule.statusDisabled}</span>
      {/if}
    </button>
  </div>
  <div class="header-actions">
    {#if isDirty}
      <div class="btn-group">
        <button class="btn btn-ghost" onclick={revertConfig}
          >{tr.actions.revert}</button
        >
        <button class="btn btn-success" onclick={saveConfig}
          >{tr.actions.save}</button
        >
      </div>
    {/if}
    <button
      class="btn btn-primary"
      onclick={handleAddSchedule}
      title={tr.schedule.addSchedule}
    >
      <IconPlus size={16} />
      {tr.schedule.addSchedule}
    </button>
  </div>
</div>

<div class="page-content">
  <ScheduleList onplay={startScheduledPlayback} {newScheduleId} />
</div>

<ConfigPanel />
<ConfirmDialog />

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    gap: 12px;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .page-content {
    flex: 1;
    overflow-y: auto;
  }
</style>
