import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { loadConfig, configStore, saveConfig, revertConfig } from "$lib/stores/config.svelte.js";
import { setPlaybackState, setSchedulerStatus } from "$lib/stores/playback.svelte.js";
import { showConfirm } from "$lib/stores/ui.svelte.js";
import { t } from "$lib/i18n/index.svelte.js";
import { getFileName, getMediaType } from "$lib/utils/thumbnail.js";

let initialization: Promise<void> | null = null;

// Owned by the app layout, so navigation never detaches scheduler/queue listeners.
export function initializeScheduledPlayback(): Promise<void> {
  return initialization ??= initialize();
}

async function initialize(): Promise<void> {
  const unlisteners: UnlistenFn[] = [];
  async function register(promise: Promise<UnlistenFn>) {
    unlisteners.push(await promise);
  }
  try {
    await register(listen<{ scheduleId: string }>("scheduler:play", ({ payload }) => {
      void startScheduledPlayback(payload.scheduleId, true).catch(console.error);
    }));
    await register(listen<PlaybackResult>("playback:ended", ({ payload }) => {
      void advancePlaybackQueue(payload).catch(console.error);
    }));
    await register(listen<PlaybackResult>("playback:started", ({ payload }) => {
      if (payload.sessionId === currentSessionId && payload.sequence === currentSequence) {
        reportPlayback();
      }
    }));
    await register(listen("playback:pause", () => setPlaybackState({ status: "paused" })));
    await register(listen("playback:resume", () => setPlaybackState({ status: "playing" })));
    await register(listen<{ reset?: boolean }>("playback:stop", ({ payload }) => {
      // Reset has already cleared the queue; its media-only stop must not
      // invalidate the pending restart when IPC delivery is delayed.
      if (payload?.reset) return;
      clearPlayback();
      void invoke("close_mini_player").catch(console.error);
    }));
    await register(listen("playback:reset", () => {
      if (!scheduleIdForReset || !isPlayingSchedule) return;
      const restartId = scheduleIdForReset;
      const scheduled = scheduledForReset;
      clearPlayback();
      const resetSequence = currentSequence;
      // Stop the old media before reopening. A subsequent start/stop cancels this reset.
      void emit("playback:stop", { reset: true }).then(() => {
        setTimeout(() => {
          if (currentSessionId === null && currentSequence === resetSequence) {
            void startScheduledPlayback(restartId, scheduled).catch(console.error);
          }
        }, 500);
      }).catch(console.error);
    }));
    await register(listen<{ status: "active" | "paused" }>("tray:status-changed", ({ payload }) => {
      setSchedulerStatus(payload.status);
    }));
    await register(listen<{ scheduleId: string; minutesBefore: number }>("scheduler:notify", ({ payload }) => {
      const schedule = configStore.savedSchedules.find((s) => s.id === payload.scheduleId);
      if (!schedule) return;
      void import("@tauri-apps/plugin-notification").then(({ sendNotification }) => {
        sendNotification({
          title: t().playback.nowPlaying,
          body: `Jadwal ${schedule.time} akan diputar dalam ${payload.minutesBefore} menit`,
        });
      }).catch(console.error);
    }));
    const appWindow = getCurrentWindow();
    await register(appWindow.listen("tauri://close-requested", async () => {
      if (configStore.isDirty) {
        showConfirm(t().unsaved.title, t().unsaved.message,
          async () => { await saveConfig(); await appWindow.hide(); },
          async () => { revertConfig(); await appWindow.hide(); });
      } else {
        await appWindow.hide();
      }
    }));
    // Do not arm Rust until every playback listener is installed.
    setSchedulerStatus(await invoke<"active" | "paused">("get_scheduler_status"));
    await loadConfig();
  } catch (error) {
    unlisteners.forEach((unlisten) => unlisten());
    initialization = null;
    throw error;
  }
}

function clearPlayback() {
  currentSessionId = null;
  currentSequence++;
  isPlayingSchedule = false;
  advancing = false;
  playQueue = [];
  queueIndex = 0;
  scheduleLoopRemaining = 0;
  scheduleLoopCountInit = 0;
  scheduleIdForReset = null;
  activeScheduleName = "";
  setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
}

function reportPlayback(error?: string) {
  if (!scheduleIdForReset) return;
  void invoke("report_playback", {
    scheduleId: scheduleIdForReset,
    path: playQueue[queueIndex]?.path ?? null,
    error: error ?? null,
  }).catch(console.error);
}

type PlaybackResult = {
  sessionId: string;
  sequence: number;
  error?: string;
};

type MiniPlayerReady = {
  requestId: string;
};

let playQueue: {
  path: string;
  volume: number;
  loopCount: number;
  type: "video" | "audio";
}[] = [];
let queueIndex = 0;
let loopRemaining = 0;
let scheduleLoopRemaining = 0;
let scheduleLoopCountInit = 0; // original schedule loop count (0 = infinite)
let isPlayingSchedule = false; // true while a schedule playlist is active
let advancing = false; // guard against re-entrant advancePlaybackQueue
let currentSessionId: string | null = null;
let currentSequence = 0;
let scheduleIdForReset: string | null = null;
let scheduledForReset = false;
let activeScheduleName = ""; // display name sent to MiniPlayer (name or time fallback)

async function waitForMiniPlayer(sessionId: string): Promise<boolean> {
  const requestId = crypto.randomUUID();
  let unlisten: UnlistenFn | null = null;
  let requestTimer: ReturnType<typeof setInterval> | null = null;
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let settled = false;

  return new Promise<boolean>(async (resolve, reject) => {
    const finish = (ready: boolean) => {
      if (settled) return;
      settled = true;
      if (requestTimer) clearInterval(requestTimer);
      if (timeout) clearTimeout(timeout);
      unlisten?.();
      resolve(ready);
    };

    try {
      unlisten = await listen<MiniPlayerReady>("mini-player:ready", ({ payload }) => {
        if (payload.requestId === requestId) finish(true);
      });

      if (currentSessionId !== sessionId) {
        finish(false);
        return;
      }

      const requestReady = () => {
        if (currentSessionId !== sessionId) {
          finish(false);
          return;
        }
        emit("mini-player:ready-request", { requestId }).catch(() => {});
      };
      requestTimer = setInterval(requestReady, 100);
      timeout = setTimeout(() => finish(false), 15000);
      requestReady();
    } catch (error) {
      if (requestTimer) clearInterval(requestTimer);
      if (timeout) clearTimeout(timeout);
      unlisten?.();
      reject(error);
    }
  });
}

export async function startScheduledPlayback(scheduleId: string, scheduled = false) {
  const schedules = scheduled ? configStore.savedSchedules : configStore.schedules;
  const schedule = schedules.find((s) => s.id === scheduleId);
  if (!schedule || schedule.media.length === 0) return;

  const sessionId = crypto.randomUUID();
  currentSessionId = sessionId;
  currentSequence = 0;
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
  scheduleLoopCountInit = lc === 0 ? 0 : Math.max(1, lc);
  isPlayingSchedule = true;
  advancing = false;
  scheduleIdForReset = scheduleId;
  scheduledForReset = scheduled;
  activeScheduleName = schedule.name?.trim() || schedule.time || "";

  setPlaybackState({
    status: "playing",
    scheduleId,
    mediaIndex: 0,
    currentLoop: 0,
  });
  try {
    await invoke("open_mini_player");
    const ready = await waitForMiniPlayer(sessionId);
    if (currentSessionId !== sessionId) return;
    if (!ready) throw new Error("Mini-player did not become ready within 15 seconds");
    await playQueueItem(0);
  } catch (error) {
    console.error("[Scheduler] Failed to start playback:", error);
    if (currentSessionId === sessionId) {
      reportPlayback(String(error));
      clearPlayback();
      await invoke("close_mini_player").catch(console.error);
    }
  }
}

async function playQueueItem(index: number) {
  const item = playQueue[index];
  const sessionId = currentSessionId;
  if (!item || !sessionId) return;
  const sequence = ++currentSequence;
  
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
  // Calculate current/total schedule-level loops for the MiniPlayer display
  const totalLoopsDisplay = scheduleLoopCountInit;  // 0 = infinite
  const currentLoopNum = totalLoopsDisplay > 0
    ? (totalLoopsDisplay - scheduleLoopRemaining + 1)
    : 1;
  await emit("playback:start", {
    sessionId,
    sequence,
    path: item.path,
    type: item.type,
    volume: item.volume,
    playlist,
    currentIndex: index,
    currentLoop: currentLoopNum,
    totalLoops: totalLoopsDisplay,
    scheduleName: activeScheduleName,
  });
}

async function advancePlaybackQueue(result: PlaybackResult) {
  // Guard: ignore stale or duplicate 'ended' events
  if (!isPlayingSchedule || playQueue.length === 0) return;
  if (result.sessionId !== currentSessionId || result.sequence !== currentSequence) return;
  if (advancing) return;
  advancing = true;

  if (result.error) {
    console.error("[Schedules] Media playback failed:", result.error);
    reportPlayback(result.error);
  }

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
        const finishedSessionId = currentSessionId;
        isPlayingSchedule = false;
        playQueue = [];
        queueIndex = 0;
        scheduleLoopRemaining = 0;
        setPlaybackState({ status: "idle", scheduleId: null, mediaPath: null });
        // Keep the WebView alive briefly so the OS audio buffer can drain.
        await new Promise((resolve) => setTimeout(resolve, 400));
        if (currentSessionId === finishedSessionId && !isPlayingSchedule) {
          currentSessionId = null;
          invoke("close_mini_player").catch(() => {});
        }
      }
    }
  } finally {
    if (result.sessionId === currentSessionId) advancing = false;
  }
}
