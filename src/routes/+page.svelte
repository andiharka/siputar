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
  } from "$lib/stores/config.svelte.js";
  import {
    playbackStore,
    setPlaybackState,
    setSchedulerStatus,
  } from "$lib/stores/playback.svelte.js";
  import { uiStore, openSettings, showConfirm } from "$lib/stores/ui.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { getFileName } from "$lib/utils/thumbnail.js";
  import ScheduleList from "$lib/components/ScheduleList.svelte";
  import ConfigPanel from "$lib/components/ConfigPanel.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import {
    IconPlayerPause,
    IconPlayerPlay,
    IconSettings,
  } from "@tabler/icons-svelte";

  const tr = $derived(t());
  const isDirty = $derived(configStore.isDirty);
  const schedulerStatus = $derived(playbackStore.schedulerStatus);

  onMount(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      await loadConfig();

      // Intercept window close: show unsaved-changes dialog if dirty
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
      await listen("playback:stop", () => {
        setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
        invoke("close_mini_player").catch(() => {});
      });
    })(); // end async IIFE

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

    setPlaybackState({
      status: "playing",
      scheduleId,
      mediaIndex: 0,
      currentLoop: 0,
    });
    await invoke("open_mini_player").catch(() => {});
    setTimeout(() => playQueueItem(0), 800);
  }

  async function playQueueItem(index: number) {
    const { emit } = await import("@tauri-apps/api/event");
    const item = playQueue[index];
    if (!item) return;
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
      setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
      invoke("close_mini_player").catch(() => {});
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
</script>

<svelte:head><title>Disperpusip Bersuara</title></svelte:head>

<div class="app-shell">
  <header class="header">
    <div class="header-left">
      <h1 class="app-title" style="font-size: 1.25rem;">{tr.app.name}</h1>
      <span
        class="badge"
        style="font-size: 1rem;"
        class:badge-active={schedulerStatus === "active"}
        class:badge-paused={schedulerStatus === "paused"}
        >{schedulerStatus === "active"
          ? tr.status.active
          : tr.status.paused}</span
      >
    </div>
    <div class="header-actions">
      {#if isDirty}
        <button class="btn btn-ghost" onclick={revertConfig}
          >{tr.actions.revert}</button
        >
        <button class="btn btn-primary" onclick={saveConfig}
          >{tr.actions.save}</button
        >
      {/if}
      <button
        class="btn btn-ghost btn-icon"
        onclick={handleToggleScheduler}
        title={schedulerStatus === "active"
          ? "Jeda semua jadwal"
          : "Aktifkan semua jadwal"}
        >{#if schedulerStatus === "active"}<IconPlayerPause
            size={16}
          />{:else}<IconPlayerPlay size={16} />{/if}</button
      >
      <button
        class="btn btn-ghost btn-icon"
        onclick={openSettings}
        title={tr.settings.title}><IconSettings size={16} /></button
      >
    </div>
  </header>

  <main class="main-scroll">
    <ScheduleList onplay={startScheduledPlayback} />
  </main>

  <footer class="footer">
    &copy; 2026 Andi &ndash; Disperpusip Jawa Timur. All rights reserved.
  </footer>

  <ConfigPanel />
  <ConfirmDialog />
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--color-bg);
  }
  .header {
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
  .app-title {
    font-size: 16px;
    font-weight: 700;
    white-space: nowrap;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .main-scroll {
    flex: 1;
    overflow-y: auto;
  }
  .footer {
    padding: 8px 20px;
    text-align: center;
    font-size: 11px;
    color: var(--color-text-muted);
    border-top: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }
</style>
