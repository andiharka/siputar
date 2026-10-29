<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke } from "@tauri-apps/api/core";
  import {
    playbackStore,
    setPlaybackState,
  } from "$lib/stores/playback.svelte.js";
  import { getFileName, getMediaDuration } from "$lib/utils/thumbnail.js";
  import { warmupAudioDevice } from "$lib/utils/audioWarmup.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { connectPlayerPreferences } from "$lib/playerPreferences.js";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    IconPlayerPause,
    IconPlayerPlay,
    IconPlayerStop,
    IconMusic,
    IconVideo,
    IconRefresh,
    IconRepeat,
    IconAlertCircle,
    IconX,
  } from "@tabler/icons-svelte";

  console.log("[MiniPlayer] Component initializing...");

  let videoEl = $state<HTMLVideoElement | null>(null);
  let audioEl = $state<HTMLAudioElement | null>(null);
  let activeType = $state<"video" | "audio" | null>(null);

  type PlaylistItem = {
    name: string;
    type: string;
    path: string;
    loopCount: number;
  };
  let playlist = $state<PlaylistItem[]>([]);
  let currentIndex = $state(0);
  let durations = $state<Record<string, number>>({});
  let playlistEl = $state<HTMLElement | null>(null);

  let unlisteners: UnlistenFn[] = [];
  let activeSessionId: string | null = null;
  let activeSequence = 0;
  let activeAssetUrl: string | null = null;
  let completedKey: string | null = null;
  let startedKey: string | null = null;

  // Progress bar state
  let currentTime = $state(0);
  let duration = $state(0);
  let loading = $state(false);
  let buffering = $state(false);
  let reducedMotion = $state(false);
  let failedFile = $state("");

  // Loop info from scheduler
  let loopCurrent = $state(0);
  let loopTotal = $state(0);

  // Schedule name shown as the title (from scheduler, fallback to time)
  let scheduleTitle = $state("");

  const pb = $derived(playbackStore.state);
  const fileName = $derived(
    playlist[currentIndex]?.path === pb.mediaPath
      ? playlist[currentIndex].name
      : pb.mediaPath ? getFileName(pb.mediaPath) : "",
  );
  const tr = $derived(t());
  const titleText = $derived(scheduleTitle || (pb.status === "idle" ? tr.playback.readyTitle : tr.playback.nowPlaying));
  const busy = $derived(loading || buffering);
  const statusText = $derived(
    pb.status === "idle" ? tr.playback.ready
      : pb.status === "paused" ? tr.playback.paused
      : busy ? tr.playback.loading : tr.playback.playing,
  );
  const canSeek = $derived(pb.status !== "idle" && !loading && duration > 0);
  const progress = $derived(duration > 0 ? Math.min(100, currentTime / duration * 100) : 0);
  const trackPosition = $derived(tr.playback.trackPosition
    .replace("{current}", String(currentIndex + 1))
    .replace("{total}", String(playlist.length)));

  function activeEl(): HTMLVideoElement | HTMLAudioElement | null {
    if (activeType === "video") return videoEl;
    if (activeType === "audio") return audioEl;
    return null;
  }

  async function loadDurationsDeferred(
    items: PlaylistItem[],
    skipIndex: number = -1,
    shouldContinue: () => boolean = () => true,
  ) {
    for (let i = 0; i < items.length; i++) {
      if (!shouldContinue()) return;
      if (i === skipIndex) continue;
      const item = items[i];
      if (durations[item.path] !== undefined) continue;

      // Small throttle interval so background metadata probing never saturates disk/IPC while playing
      await new Promise((resolve) => setTimeout(resolve, 300));
      if (!shouldContinue()) return;

      const d = await getMediaDuration(
        item.path,
        item.path,
        item.type as "video" | "audio",
      );
      if (!shouldContinue()) return;
      if (d > 0) {
        durations = { ...durations, [item.path]: d };
      }
    }
  }

  async function scrollToActive() {
    await tick();
    playlistEl
      ?.querySelector<HTMLElement>(".playlist-item.active")
      ?.scrollIntoView({ block: "nearest", behavior: reducedMotion ? "instant" : "smooth" });
  }

  function isCurrent(sessionId: string, sequence: number): boolean {
    return activeSessionId === sessionId && activeSequence === sequence;
  }

  function reportStarted(sessionId: string, sequence: number) {
    const key = `${sessionId}:${sequence}`;
    if (!isCurrent(sessionId, sequence) || startedKey === key) return;
    startedKey = key;
    void emit("playback:started", { sessionId, sequence }).catch(console.error);
  }

  function formatTime(s: number): string {
    if (!isFinite(s) || s < 0) return "0:00";
    const mins = Math.floor(s / 60);
    const secs = Math.floor(s % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  function isLooped(): boolean {
    return loopTotal > 1 || loopTotal === 0;
  }

  function formatProgressText(): string {
    if (loopTotal > 1) {
      return `${tr.playback.loop} ${loopCurrent}/${loopTotal}`;
    }
    if (loopTotal === 0) {
      return `${tr.playback.loop} ∞`;
    }
    return "";
  }

  function waitUntilPlayable(
    el: HTMLMediaElement,
    sessionId: string,
    sequence: number,
  ): Promise<void> {
    if (el.readyState >= HTMLMediaElement.HAVE_FUTURE_DATA) return Promise.resolve();

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => finish(new Error("Timed out loading media")), 10000);
      const onCanPlay = () => finish();
      const onError = () => finish(new Error(el.error?.message || "Failed to load media"));

      function finish(error?: Error) {
        clearTimeout(timeout);
        el.removeEventListener("canplay", onCanPlay);
        el.removeEventListener("error", onError);
        if (!isCurrent(sessionId, sequence)) resolve();
        else if (error) reject(error);
        else resolve();
      }

      el.addEventListener("canplay", onCanPlay, { once: true });
      el.addEventListener("error", onError, { once: true });
    });
  }



  onMount(() => {
    const disconnect = connectPlayerPreferences();
    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotion = () => { reducedMotion = preference.matches; };
    updateMotion();
    preference.addEventListener("change", updateMotion);
    return () => {
      disconnect();
      preference.removeEventListener("change", updateMotion);
    };
  });

  onMount(async () => {
    console.log("[MiniPlayer] Component mounted");

    // Pre-warm audio subsystem on mount so WASAPI is initialized early
    warmupAudioDevice().catch(() => {});

    // Add F12 keyboard shortcut for DevTools
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "F12") {
        e.preventDefault();
        console.log("[MiniPlayer] F12 pressed - toggling DevTools");
        invoke("toggle_mini_player_devtools").catch((err) => {
          console.error("[MiniPlayer] Failed to toggle DevTools:", err);
        });
      }
    };
    window.addEventListener("keydown", handleKeyDown);

    // Store cleanup function for onDestroy
    unlisteners.push(() =>
      window.removeEventListener("keydown", handleKeyDown),
    );

    unlisteners.push(
      await listen<{
        sessionId: string;
        sequence: number;
        path: string;
        type: "video" | "audio";
        volume: number;
        playlist?: PlaylistItem[];
        currentIndex?: number;
        currentLoop?: number;
        totalLoops?: number;
        scheduleName?: string;
      }>("playback:start", async ({ payload }) => {
        console.log("[MiniPlayer] playback:start event received:", payload);
        activeSessionId = payload.sessionId;
        activeSequence = payload.sequence;
        completedKey = null;
        startedKey = null;
        activeType = payload.type;
        loading = true;
        buffering = false;
        currentTime = 0;
        duration = 0;
        loopCurrent = payload.currentLoop ?? 0;
        loopTotal = payload.totalLoops ?? 0;
        scheduleTitle = payload.scheduleName ?? "";
        if (payload.playlist) {
          playlist = payload.playlist;
        }
        if (payload.currentIndex !== undefined)
          currentIndex = payload.currentIndex;
        setPlaybackState({
          status: "playing",
          mediaPath: payload.path,
          mediaType: payload.type,
        });

        // Wait for DOM to update so videoEl is bound before playing
        await tick();
        if (!isCurrent(payload.sessionId, payload.sequence)) return;

        const el = payload.type === "video" ? videoEl : audioEl;
        if (el) {
          videoEl?.pause();
          audioEl?.pause();
          const assetUrl = payload.path.startsWith("/media/")
            ? payload.path
            : convertFileSrc(payload.path);
          console.log("[MiniPlayer] Loading media:", payload.path);
          console.log("[MiniPlayer] Converted URL:", assetUrl);
          el.src = assetUrl;
          activeAssetUrl = el.src;
          el.volume = payload.volume;
          el.load();

          try {
            await waitUntilPlayable(el, payload.sessionId, payload.sequence);
            if (!isCurrent(payload.sessionId, payload.sequence)) return;

            // Ensure Windows audio engine/WASAPI is awake and primed before playing
            await warmupAudioDevice();
            if (!isCurrent(payload.sessionId, payload.sequence)) return;

            // A pause received while loading must also prevent autoplay.
            loading = false;
            if (pb.status === "playing") {
              await el.play();
              if (!isCurrent(payload.sessionId, payload.sequence)) return;
              reportStarted(payload.sessionId, payload.sequence);
            }

            // Populate active track duration immediately if already loaded
            if (el.duration && isFinite(el.duration)) {
              durations = { ...durations, [payload.path]: el.duration };
            }

            // Defer probing durations for the rest of the playlist in background
            if (payload.playlist && payload.playlist.length > 1) {
              loadDurationsDeferred(
                payload.playlist,
                payload.currentIndex ?? 0,
                () => isCurrent(payload.sessionId, payload.sequence),
              );
            }
          } catch (err) {
            console.error("[MiniPlayer] Failed to play media:", err);
            finishPlayback(
              payload.sessionId,
              payload.sequence,
              err instanceof Error ? err.message : String(err),
            );
          }
        } else {
          console.error(
            "[MiniPlayer] Media element not found for type:",
            payload.type,
          );
          finishPlayback(
            payload.sessionId,
            payload.sequence,
            `Media element not found for type: ${payload.type}`,
          );
        }
        scrollToActive();
      }),
      await listen("playback:pause", () => {
        console.log("[MiniPlayer] playback:pause event received");
        setPlaybackState({ status: "paused" });
        activeEl()?.pause();
      }),
      await listen("playback:resume", () => {
        console.log("[MiniPlayer] playback:resume event received");
        setPlaybackState({ status: "playing" });
        if (loading) return;
        const sessionId = activeSessionId;
        const sequence = activeSequence;
        activeEl()
          ?.play()
          .then(() => {
            if (sessionId && isCurrent(sessionId, sequence)) {
              reportStarted(sessionId, sequence);
            }
          })
          .catch((err) => {
            if (sessionId && isCurrent(sessionId, sequence)) {
              console.error("[MiniPlayer] Failed to resume:", err);
              finishPlayback(sessionId, sequence, String(err));
            }
          });
      }),
      await listen("playback:stop", () => {
        console.log("[MiniPlayer] playback:stop event received");
        setPlaybackState({ status: "idle", mediaPath: null, mediaType: null });
        activeType = null;
        activeSessionId = null;
        activeSequence++;
        activeAssetUrl = null;
        completedKey = null;
        playlist = [];
        currentIndex = 0;
        durations = {};
        currentTime = 0;
        duration = 0;
        loading = false;
        buffering = false;
        failedFile = "";
        loopCurrent = 0;
        loopTotal = 0;
        scheduleTitle = "";
        // Clear media elements without triggering load() error
        if (videoEl) {
          videoEl.pause();
          videoEl.removeAttribute("src");
        }
        if (audioEl) {
          audioEl.pause();
          audioEl.removeAttribute("src");
        }
      }),
      await listen<{ requestId: string }>("mini-player:ready-request", ({ payload }) => {
        // Pre-warm audio device while handshaking with schedules page
        warmupAudioDevice().catch(() => {});
        emit("mini-player:ready", { requestId: payload.requestId });
      }),
    );
    console.log("[MiniPlayer] Ready for playback requests");
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });

  function finishPlayback(sessionId: string, sequence: number, error?: string) {
    if (!isCurrent(sessionId, sequence)) return;
    const key = `${sessionId}:${sequence}`;
    if (completedKey === key) return;
    completedKey = key;
    loading = false;
    buffering = false;
    if (error) failedFile = fileName;
    console.log(error ? "[MiniPlayer] Media failed" : "[MiniPlayer] Media ended");
    emit("playback:ended", { sessionId, sequence, error });
  }

  function handleEnded(e: Event) {
    if (e.target !== activeEl()) return;
    finishPlayback(activeSessionId ?? "", activeSequence);
  }

  function handlePause() {
    emit("playback:pause", {});
  }
  function handleResume() {
    emit("playback:resume", {});
  }
  function handleStop() {
    emit("playback:stop", {});
  }
  function handleReset() {
    emit("playback:reset", {});
  }

  function handleMediaError(e: Event) {
    const target = e.target as HTMLMediaElement;
    console.error("[MiniPlayer] Media error:", {
      error: target.error,
      src: target.src,
      networkState: target.networkState,
      readyState: target.readyState,
    });
    if (target !== activeEl() || (activeAssetUrl && target.src !== activeAssetUrl)) return;
    // Skip to next item if media fails to load
    finishPlayback(
      activeSessionId ?? "",
      activeSequence,
      target.error?.message || `Media error ${target.error?.code ?? "unknown"}`,
    );
  }

  function handleMediaLoaded(e: Event) {
    const target = e.target as HTMLMediaElement;
    console.log("[MiniPlayer] Media loaded successfully:", target.src);
  }

  function handleTimeUpdate() {
    const el = activeEl();
    if (el && el.duration && isFinite(el.duration)) {
      currentTime = el.currentTime;
      duration = el.duration;
    }
  }

  function seekTo(time: number) {
    const el = activeEl();
    if (!el || !canSeek) return;
    currentTime = Math.max(0, Math.min(duration, time));
    el.currentTime = currentTime;
  }

  function handleSeek(event: Event) {
    seekTo(Number((event.target as HTMLInputElement).value));
  }

  function handleSeekKey(event: KeyboardEvent) {
    const targets: Record<string, number> = {
      ArrowLeft: currentTime - 5,
      ArrowDown: currentTime - 5,
      ArrowRight: currentTime + 5,
      ArrowUp: currentTime + 5,
      Home: 0,
      End: duration,
      PageDown: currentTime - duration * 0.1,
      PageUp: currentTime + duration * 0.1,
    };
    if (event.key in targets) {
      event.preventDefault();
      seekTo(targets[event.key]);
    }
  }

  function handleWaiting(event: Event) {
    if (event.target === activeEl()) buffering = true;
  }

  function handlePlaying(event: Event) {
    if (event.target === activeEl()) buffering = false;
  }

  function handleMediaLoadedMetadata() {
    const el = activeEl();
    if (el && el.duration && isFinite(el.duration)) {
      duration = el.duration;
      if (pb.mediaPath) {
        durations = { ...durations, [pb.mediaPath]: el.duration };
      }
    }
  }

  function loopLabel(n: number): string {
    if (n === 0) return "∞";
    if (n === 1) return "";
    return `×${n}`;
  }
</script>

<svelte:head><title>{tr.nav.miniPlayer} · {tr.app.name}</title></svelte:head>

<main class="player" class:is-idle={pb.status === "idle"} class:has-video={pb.mediaType === "video"}>
  <header class="player-header" data-tauri-drag-region>
    <div class="heading" data-tauri-drag-region>
      <span class="app-label" data-tauri-drag-region>{tr.app.name} / {tr.nav.miniPlayer}</span>
      <h1 title={titleText} data-tauri-drag-region>{titleText}</h1>
    </div>
    <span class="player-status" class:playing={pb.status === "playing" && !busy} role="status">
      <span class="status-dot" aria-hidden="true"></span>{statusText}
    </span>
  </header>

  <!-- Keep both elements mounted; WebView2 can lose bindings while swapping nodes. -->
  <div class="media-area" data-tauri-drag-region>
    <video
      bind:this={videoEl}
      class:hidden={pb.mediaType !== "video"}
      preload="auto"
      onended={handleEnded}
      onerror={handleMediaError}
      onloadeddata={handleMediaLoaded}
      ontimeupdate={handleTimeUpdate}
      onloadedmetadata={handleMediaLoadedMetadata}
      onwaiting={handleWaiting}
      onplaying={handlePlaying}
    ></video>
    <audio
      bind:this={audioEl}
      preload="auto"
      onended={handleEnded}
      onerror={handleMediaError}
      onloadeddata={handleMediaLoaded}
      ontimeupdate={handleTimeUpdate}
      onloadedmetadata={handleMediaLoadedMetadata}
      onwaiting={handleWaiting}
      onplaying={handlePlaying}
    ></audio>
    <div class="video-drag-overlay" data-tauri-drag-region></div>
  </div>

  <section class="track-details" aria-label={tr.playback.nowPlaying}>
    {#if pb.mediaType !== "video"}
      <div class="audio-art" class:idle-art={pb.status === "idle"} aria-hidden="true">
        <IconMusic size={pb.status === "idle" ? 32 : 24} stroke={1.5} />
      </div>
    {/if}
    <div class="track-copy">
      {#if pb.status !== "idle"}
        <div class="track-meta">
          <span>{pb.mediaType === "video" ? tr.playback.video : tr.playback.audio}</span>
          {#if playlist.length > 0}<span>{trackPosition}</span>{/if}
        </div>
      {/if}
      {#key pb.mediaPath}
        <h2 title={fileName}>{fileName || tr.playback.readyTitle}</h2>
      {/key}
      {#if pb.status === "idle"}<p class="idle-hint">{tr.playback.readyHint}</p>{/if}
    </div>
  </section>

  <section class="transport" aria-label={tr.playback.controls}>
    <div class="timeline">
      <input
        class="seek"
        type="range"
        min="0"
        max={duration || 1}
        step="0.1"
        value={currentTime}
        style="--progress: {progress}%"
        aria-label={tr.playback.seek}
        aria-valuetext={`${formatTime(currentTime)} / ${duration > 0 ? formatTime(duration) : tr.playback.unknownDuration}`}
        disabled={!canSeek}
        oninput={handleSeek}
        onkeydown={handleSeekKey}
      />
      <div class="time-labels">
        <span>{formatTime(currentTime)}</span>
        <span>{duration > 0 ? formatTime(duration) : "--:--"}</span>
      </div>
    </div>
    <div class="buttons">
      <button class="ctrl-btn secondary" onclick={handleReset} disabled={pb.status === "idle"}
        aria-label={tr.playback.restartSchedule} title={tr.playback.restartSchedule}>
        <IconRefresh size={18} /><span>{tr.playback.reset}</span>
      </button>
      <button class="ctrl-btn primary" onclick={pb.status === "playing" ? handlePause : handleResume}
        disabled={pb.status === "idle"}
        aria-label={pb.status === "playing" ? tr.playback.pause : tr.playback.resume}
        title={pb.status === "playing" ? tr.playback.pause : tr.playback.resume}>
        {#if pb.status === "playing"}<IconPlayerPause size={24} fill="currentColor" />
        {:else}<IconPlayerPlay size={24} fill="currentColor" />{/if}
      </button>
      <button class="ctrl-btn secondary stop" onclick={handleStop} disabled={pb.status === "idle"}
        aria-label={tr.playback.stop} title={tr.playback.stop}>
        <IconPlayerStop size={18} /><span>{tr.playback.stop}</span>
      </button>
    </div>
  </section>

  {#if failedFile}
    <div class="failure" role="status">
      <IconAlertCircle size={16} />
      <span>{tr.playback.skippedFile.replace("{name}", failedFile)}</span>
      <button onclick={() => { failedFile = ""; }} aria-label={tr.actions.close}><IconX size={14} /></button>
    </div>
  {/if}

  {#if playlist.length > 0}
    <section class="queue" aria-label={tr.playback.queue}>
      <div class="queue-header">
        <h3>{tr.playback.queue} <span>{playlist.length}</span></h3>
        {#if isLooped()}
          <span class="loop-label"><IconRepeat size={13} />{formatProgressText()}</span>
        {/if}
      </div>
      <ol class="playlist" bind:this={playlistEl}>
        {#each playlist as item, i}
          <li class="playlist-item" class:active={i === currentIndex} aria-current={i === currentIndex ? "true" : undefined}>
            <span class="item-number" aria-hidden="true">
              {#if i === currentIndex}
                <span class="equalizer" class:running={pb.status === "playing" && !busy}><i></i><i></i><i></i></span>
              {:else}{String(i + 1).padStart(2, "0")}{/if}
            </span>
            <span class="item-copy">
              <span class="item-name" title={item.name}>{item.name}</span>
              <span class="item-type">
                {#if item.type === "video"}<IconVideo size={12} />{tr.playback.video}
                {:else}<IconMusic size={12} />{tr.playback.audio}{/if}
                {#if loopLabel(item.loopCount)}<span class="item-loop">{loopLabel(item.loopCount)}</span>{/if}
              </span>
            </span>
            <span class="item-dur">{durations[item.path] > 0 ? formatTime(durations[item.path]) : "--:--"}</span>
          </li>
        {/each}
      </ol>
    </section>
  {/if}
</main>

<style>
  .player {
    height: 100dvh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--color-text);
    background: var(--color-surface);
  }
  .player-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 18px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .heading { min-width: 0; }
  .app-label { display: block; font-size: 10px; color: var(--color-text-muted); margin-bottom: 3px; }
  h1 { font-size: 14px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .player-status { display: inline-flex; align-items: center; gap: 5px; font-size: 11px; color: var(--color-text-muted); flex-shrink: 0; }
  .status-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
  .player-status.playing { color: var(--color-primary); }
  .media-area { height: 0; overflow: hidden; position: relative; flex-shrink: 0; background: #111; }
  .has-video .media-area { height: min(180px, 35dvh); }
  video { display: block; width: 100%; height: 100%; object-fit: contain; pointer-events: none; }
  video.hidden, audio { display: none; }
  .video-drag-overlay { position: absolute; inset: 0; }
  .track-details { display: flex; align-items: center; gap: 12px; padding: 18px 18px 8px; flex-shrink: 0; }
  .audio-art { display: grid; place-items: center; width: 52px; height: 56px; border-radius: var(--radius-md); background: var(--color-surface-2); color: var(--color-primary); flex-shrink: 0; }
  .track-copy { min-width: 0; }
  .track-meta { display: flex; gap: 10px; font-size: 11px; color: var(--color-text-muted); margin-bottom: 4px; }
  h2 { font-size: 17px; line-height: 1.35; font-weight: 600; letter-spacing: -0.3px; overflow: hidden; display: -webkit-box; line-clamp: 2; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow-wrap: anywhere; animation: track-enter 180ms ease-out; }
  .has-video .track-details { padding-top: 12px; }
  .has-video h2 { font-size: 15px; line-clamp: 1; -webkit-line-clamp: 1; }
  .transport { padding: 0 18px 16px; flex-shrink: 0; }
  .timeline { margin-bottom: 8px; }
  .seek { display: block; width: 100%; height: 24px; appearance: none; -webkit-appearance: none; background: transparent; cursor: pointer; outline: none; }
  .seek::-webkit-slider-runnable-track { height: 4px; border-radius: 8px; background: linear-gradient(to right, var(--color-primary) var(--progress), var(--color-border) var(--progress)); }
  .seek::-webkit-slider-thumb { appearance: none; -webkit-appearance: none; height: 12px; width: 12px; margin-top: -4px; border-radius: 50%; background: var(--color-primary); }
  .seek::-moz-range-track { height: 4px; background: var(--color-border); border-radius: 8px; }
  .seek::-moz-range-progress { height: 4px; background: var(--color-primary); }
  .seek::-moz-range-thumb { height: 12px; width: 12px; border: 0; border-radius: 50%; background: var(--color-primary); }
  .seek:disabled { cursor: default; opacity: 0.4; }
  .seek:focus-visible { outline: 2px solid var(--color-primary); outline-offset: 2px; border-radius: 4px; }
  .time-labels { display: flex; justify-content: space-between; font-size: 11px; font-variant-numeric: tabular-nums; color: var(--color-text-muted); }
  .buttons { display: flex; align-items: center; justify-content: center; gap: 20px; }
  .ctrl-btn { display: inline-flex; align-items: center; justify-content: center; border: 0; cursor: pointer; color: var(--color-text); background: transparent; transition: background 150ms ease, transform 100ms ease; }
  .ctrl-btn.primary { width: 52px; height: 52px; border-radius: 50%; background: var(--color-primary); color: var(--color-surface); flex-shrink: 0; }
  .ctrl-btn.primary:hover:not(:disabled) { background: var(--color-primary-hover); }
  .ctrl-btn.secondary { flex-direction: column; gap: 4px; width: 72px; min-height: 46px; border-radius: var(--radius-sm); color: var(--color-text-muted); font-size: 10px; }
  .ctrl-btn.secondary:hover:not(:disabled) { background: var(--color-surface-3); color: var(--color-text); }
  .ctrl-btn.stop:hover:not(:disabled) { color: var(--color-danger); }
  .ctrl-btn:active:not(:disabled) { transform: scale(0.95); }
  .ctrl-btn:focus-visible, .failure button:focus-visible { outline: 2px solid var(--color-primary); outline-offset: 3px; }
  .ctrl-btn:disabled { opacity: 0.35; cursor: not-allowed; }
  .queue { display: flex; flex-direction: column; flex: 1; min-height: 0; background: var(--color-bg); border-top: 1px solid var(--color-border); }
  .queue-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 12px 18px 8px; flex-shrink: 0; }
  h3 { font-size: 12px; font-weight: 600; }
  h3 span { margin-left: 5px; font-size: 11px; font-weight: 400; color: var(--color-text-muted); }
  .loop-label { display: inline-flex; align-items: center; gap: 4px; font-size: 10px; color: var(--color-primary); white-space: nowrap; }
  .playlist { min-height: 0; overflow-y: auto; overscroll-behavior: contain; list-style: none; padding: 0 8px 8px; scrollbar-width: thin; }
  .playlist-item { display: flex; align-items: center; gap: 10px; min-height: 52px; padding: 8px 10px; border-radius: var(--radius-sm); color: var(--color-text-muted); }
  .playlist-item.active { background: var(--color-surface-2); color: var(--color-text); }
  .item-number { display: grid; place-items: center; width: 22px; flex-shrink: 0; font-size: 11px; font-variant-numeric: tabular-nums; }
  .item-copy { display: flex; flex-direction: column; gap: 3px; min-width: 0; flex: 1; }
  .item-name { font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .active .item-name { font-weight: 600; }
  .item-type { display: flex; align-items: center; gap: 4px; font-size: 10px; color: var(--color-text-muted); }
  .item-loop { margin-left: 4px; color: var(--color-primary); }
  .item-dur { font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }
  .equalizer { display: flex; align-items: center; gap: 2px; height: 16px; color: var(--color-primary); }
  .equalizer i { display: block; width: 3px; height: 12px; border-radius: 2px; background: currentColor; transform: scaleY(0.35); }
  .equalizer.running i { animation: level 800ms ease-in-out infinite alternate; }
  .equalizer.running i:nth-child(2) { animation-delay: -400ms; }
  .equalizer.running i:nth-child(3) { animation-delay: -200ms; }
  .failure { display: flex; align-items: center; gap: 8px; padding: 8px 18px; color: var(--color-danger); background: var(--color-surface); font-size: 11px; flex-shrink: 0; }
  .failure > :global(svg) { flex-shrink: 0; }
  .failure span { flex: 1; min-width: 0; overflow-wrap: anywhere; max-height: 3em; overflow: auto; }
  .failure button { display: grid; place-items: center; border: 0; padding: 6px; color: inherit; background: transparent; cursor: pointer; }
  .is-idle .track-details { flex: 1; flex-direction: column; justify-content: center; text-align: center; padding: 24px; gap: 16px; }
  .is-idle .track-copy { max-width: 260px; }
  .idle-art { width: 72px; height: 72px; border-radius: 20px; }
  .idle-hint { font-size: 13px; line-height: 1.6; color: var(--color-text-muted); margin-top: 8px; }
  .is-idle .transport { padding-bottom: 24px; }
  @keyframes track-enter { from { opacity: 0; transform: translateY(3px); } to { opacity: 1; transform: translateY(0); } }
  @keyframes level { from { transform: scaleY(0.3); } to { transform: scaleY(1); } }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation: none !important; transition: none !important; }
  }
  @media (max-height: 480px) {
    .has-video .media-area { max-height: 34dvh; }
    .player-header { padding-top: 10px; padding-bottom: 10px; }
    .transport { padding-bottom: 10px; }
    .buttons { gap: 16px; }
    .ctrl-btn.primary { width: 46px; height: 46px; }
  }
</style>
