<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    loadConfig,
    configStore,
    saveConfig,
    revertConfig,
    addSchedule,
  } from "$lib/stores/config.svelte.js";
  import {
    playbackStore,
    setPlaybackState,
    setSchedulerStatus,
  } from "$lib/stores/playback.svelte.js";
  import { showConfirm } from "$lib/stores/ui.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { getFileName } from "$lib/utils/thumbnail.js";
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

  onMount(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      await loadConfig();

      const appWindow = getCurrentWindow();
      unlisten = await appWindow.listen("tauri://close-requested", async () => {
        if (configStore.isDirty) {
          showConfirm(
            tr.unsaved.title,
            tr.unsaved.message,
            async () => {
              await saveConfig();
              await appWindow.hide();
            },
            async () => {
              revertConfig();
              await appWindow.hide();
            },
          );
        } else {
          await appWindow.hide();
        }
      });

      await listen<{ status: string }>("tray:status-changed", ({ payload }) => {
        setSchedulerStatus(payload.status as "active" | "paused");
      });

      await listen<{ scheduleId: string }>("scheduler:play", ({ payload }) => {
        startScheduledPlayback(payload.scheduleId);
      });

      await listen<{ scheduleId: string; minutesBefore: number }>(
        "scheduler:notify",
        ({ payload }) => {
          const schedule = configStore.schedules.find(
            (s) => s.id === payload.scheduleId,
          );
          if (!schedule) return;
          import("@tauri-apps/plugin-notification").then(
            ({ sendNotification }) => {
              sendNotification({
                title: tr.playback.nowPlaying,
                body: `Jadwal ${schedule.time} akan diputar dalam ${payload.minutesBefore} menit`,
              });
            },
          );
        },
      );

      await listen("playback:ended", () => advancePlaybackQueue());
      await listen("playback:pause", () =>
        setPlaybackState({ status: "paused" }),
      );
      await listen("playback:resume", () =>
        setPlaybackState({ status: "playing" }),
      );
      // playback:stop is only emitted by MiniPlayer's stop button (manual stop)
      await listen("playback:stop", () => {
        setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
        isPlayingSchedule = false;
        advancing = false;
        playQueue = [];
        queueIndex = 0;
        scheduleLoopRemaining = 0;
        invoke("close_mini_player").catch(() => {});
      });
    })();

    return () => {
      unlisten?.();
    };
  });

  let playQueue: {
    path: string;
    volume: number;
    loopCount: number;
    type: "video" | "audio";
  }[] = [];
  let queueIndex = 0;
  let loopRemaining = 0;
  let scheduleLoopRemaining = 0;
  let isPlayingSchedule = false; // true while a schedule playlist is active
  let advancing = false; // guard against re-entrant advancePlaybackQueue
  let newScheduleId = $state<string | null>(null);

  async function startScheduledPlayback(scheduleId: string) {
    const schedule = configStore.schedules.find((s) => s.id === scheduleId);
    if (!schedule || schedule.media.length === 0) return;

    const { getMediaType } = await import("$lib/utils/thumbnail.js");
    playQueue = schedule.media.map((m) => ({
      path: m.path,
      volume: m.volume,
      loopCount: m.loopCount,
      type: getMediaType(m.path),
    }));
    queueIndex = 0;
    loopRemaining = Math.max(1, playQueue[0]?.loopCount ?? 1);
    // Schedule-level loop: 0 = infinite (-1 sentinel), otherwise the total number of passes
    const lc = schedule.loopCount ?? 1;
    scheduleLoopRemaining = lc === 0 ? -1 : Math.max(1, lc);
    isPlayingSchedule = true;
    advancing = false;

    setPlaybackState({
      status: "playing",
      scheduleId,
      mediaIndex: 0,
      currentLoop: 0,
    });
    await invoke("open_mini_player").catch(() => {});
    // Small delay to let the mini-player window initialize before sending media
    setTimeout(() => playQueueItem(0), 300);
  }

  async function playQueueItem(index: number) {
    const { emit } = await import("@tauri-apps/api/event");
    const item = playQueue[index];
    if (!item) return;
    
    // Don't call open_mini_player here — it's already opened once in
    // startScheduledPlayback(). Re-calling show()/hide() between tracks
    // triggers Windows window animations and causes stutter.
    
    setPlaybackState({
      mediaPath: item.path,
      mediaType: item.type,
      mediaIndex: index,
      currentIndex: index,
    });
    const playlist = playQueue.map((q) => ({
      name: getFileName(q.path),
      type: q.type,
      path: q.path,
      loopCount: q.loopCount,
    }));
    await emit("playback:start", {
      path: item.path,
      type: item.type,
      volume: item.volume,
      playlist,
      currentIndex: index,
    });
  }

  async function advancePlaybackQueue() {
    // Guard: ignore stale or duplicate 'ended' events
    if (!isPlayingSchedule || playQueue.length === 0) return;
    if (advancing) return;
    advancing = true;

    try {
      loopRemaining--;
      if (loopRemaining > 0) {
        await playQueueItem(queueIndex);
        return;
      }
      queueIndex++;
      if (queueIndex < playQueue.length) {
        loopRemaining = Math.max(1, playQueue[queueIndex].loopCount);
        await playQueueItem(queueIndex);
      } else {
        // Entire playlist finished — check schedule-level loop
        if (scheduleLoopRemaining === -1) {
          // Infinite loop: restart from beginning
          queueIndex = 0;
          loopRemaining = Math.max(1, playQueue[0].loopCount);
          await playQueueItem(0);
        } else if (scheduleLoopRemaining > 1) {
          // More repeats remaining: decrement and restart
          scheduleLoopRemaining--;
          queueIndex = 0;
          loopRemaining = Math.max(1, playQueue[0].loopCount);
          await playQueueItem(0);
        } else {
          // All done — clean up and hide mini-player
          isPlayingSchedule = false;
          playQueue = [];
          queueIndex = 0;
          scheduleLoopRemaining = 0;
          setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
          invoke("close_mini_player").catch(() => {});
        }
      }
    } finally {
      advancing = false;
    }
  }

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
