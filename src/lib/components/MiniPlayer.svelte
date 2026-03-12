<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    playbackStore,
    setPlaybackState,
  } from "$lib/stores/playback.svelte.js";
  import { getFileName, getMediaDuration } from "$lib/utils/thumbnail.js";
  import { formatDuration } from "$lib/utils/duration.js";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    IconPlayerPause,
    IconPlayerPlay,
    IconPlayerStop,
    IconMusic,
    IconVideo,
  } from "@tabler/icons-svelte";

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

  const pb = $derived(playbackStore.state);
  const fileName = $derived(pb.mediaPath ? getFileName(pb.mediaPath) : "");

  function activeEl(): HTMLVideoElement | HTMLAudioElement | null {
    if (activeType === "video") return videoEl;
    if (activeType === "audio") return audioEl;
    return null;
  }

  async function loadDurations(items: PlaylistItem[]) {
    for (const item of items) {
      if (durations[item.path] !== undefined) continue;
      const d = await getMediaDuration(
        item.path,
        item.path,
        item.type as "video" | "audio",
      );
      durations = { ...durations, [item.path]: d };
    }
  }

  async function scrollToActive() {
    await tick();
    playlistEl
      ?.querySelector<HTMLElement>(".playlist-item.active")
      ?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  onMount(async () => {
    unlisteners.push(
      await listen<{
        path: string;
        type: "video" | "audio";
        volume: number;
        playlist?: PlaylistItem[];
        currentIndex?: number;
      }>("playback:start", async ({ payload }) => {
        activeType = payload.type;
        if (payload.playlist) {
          playlist = payload.playlist;
          loadDurations(payload.playlist); // load async, don't await
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

        const el = payload.type === "video" ? videoEl : audioEl;
        if (el) {
          el.src = convertFileSrc(payload.path);
          el.volume = payload.volume;
          el.play().catch(console.error);
        }
        scrollToActive();
      }),
      await listen("playback:pause", () => {
        setPlaybackState({ status: "paused" });
        activeEl()?.pause();
      }),
      await listen("playback:resume", () => {
        setPlaybackState({ status: "playing" });
        activeEl()
          ?.play()
          .catch(() => {});
      }),
      await listen("playback:stop", () => {
        setPlaybackState({ status: "idle", mediaPath: null, mediaType: null });
        activeType = null;
        playlist = [];
        currentIndex = 0;
        durations = {};
        if (videoEl) {
          videoEl.src = "";
          videoEl.load();
        }
        if (audioEl) {
          audioEl.src = "";
          audioEl.load();
        }
      }),
    );
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
  });

  function handleEnded() {
    emit("playback:ended", {});
  }
  async function handlePause() {
    await emit("playback:pause", {});
  }
  async function handleResume() {
    await emit("playback:resume", {});
  }
  async function handleStop() {
    await emit("playback:stop", {});
  }

  function loopLabel(n: number): string {
    if (n === 0) return "∞";
    if (n === 1) return "";
    return `×${n}`;
  }
</script>

<div class="player" data-tauri-drag-region>
  <!-- Media area: video is behind a transparent drag overlay -->
  {#if pb.mediaType === "video"}
    <div class="media-area" data-tauri-drag-region>
      <video
        bind:this={videoEl}
        onended={handleEnded}
        style="width:100%;height:100%;object-fit:contain;pointer-events:none;"
      ></video>
      <!-- Transparent overlay so drag-region captures events above video -->
      <div class="video-drag-overlay" data-tauri-drag-region></div>
    </div>
  {:else}
    <!-- Audio: just a dark placeholder -->
    <div class="media-area audio-placeholder" data-tauri-drag-region>
      <audio bind:this={audioEl} onended={handleEnded}></audio>
      <!-- keep video bound but hidden -->
      <video bind:this={videoEl} style="display:none;"></video>
      <div class="audio-icon"><IconMusic size={32} color="yellowgreen" /></div>
    </div>
  {/if}

  <!-- Controls bar -->
  <div class="controls" data-tauri-drag-region>
    <div class="info" data-tauri-drag-region title={pb.mediaPath ?? ""}>
      <span class="status-dot" class:playing={pb.status === "playing"}></span>
      <span class="filename">{fileName || "—"}</span>
    </div>
    <div class="buttons">
      {#if pb.status === "playing"}
        <button class="ctrl-btn" onclick={handlePause} title="Pause"
          ><IconPlayerPause size={20} /></button
        >
      {:else}
        <button
          class="ctrl-btn"
          onclick={handleResume}
          title="Resume"
          disabled={pb.status === "idle"}><IconPlayerPlay size={20} /></button
        >
      {/if}
      <button
        class="ctrl-btn danger"
        onclick={handleStop}
        title="Stop"
        disabled={pb.status === "idle"}><IconPlayerStop size={20} /></button
      >
    </div>
  </div>

  <!-- Playlist -->
  {#if playlist.length > 0}
    <div class="playlist" bind:this={playlistEl}>
      {#each playlist as item, i}
        <div class="playlist-item" class:active={i === currentIndex}>
          <span class="item-icon">
            {#if item.type === "video"}<IconVideo size={12} />{:else}<IconMusic
                size={12}
              />{/if}
          </span>
          <span class="item-name" title={item.name}>{item.name}</span>
          <span class="item-meta">
            {#if durations[item.path] > 0}
              <span class="item-dur"
                >{formatDuration(durations[item.path])}</span
              >
            {/if}
            {#if loopLabel(item.loopCount)}
              <span class="item-loop">{loopLabel(item.loopCount)}</span>
            {/if}
          </span>
          {#if i === currentIndex}
            <span class="now-dot"></span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  :global(body) {
    background: #1a1a2e;
    color: #e0e0e0;
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
    overflow: hidden;
    height: 100vh;
  }

  .player {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: #1a1a2e;
  }

  .media-area {
    height: 211px;
    flex-shrink: 0;
    overflow: hidden;
    background: #000;
    position: relative;
  }
  .video-drag-overlay {
    position: absolute;
    inset: 0;
    z-index: 1;
  }
  .audio-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d0d1e;
  }
  .audio-icon {
    opacity: 1;
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    gap: 10px;
    flex-shrink: 0;
    background: #12122a;
    border-bottom: 1px solid #2a2a4a;
  }
  .info {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
    flex: 1;
  }
  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #555;
    flex-shrink: 0;
    transition: 0.2s;
  }
  .status-dot.playing {
    background: #4ade80;
  }
  .filename {
    font-size: 16px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .buttons {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
  .ctrl-btn {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #e0e0e0;
    border-radius: 6px;
    padding: 6px 10px;
    cursor: pointer;
    transition: 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .ctrl-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }
  .ctrl-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .ctrl-btn.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }

  /* Playlist */
  .playlist {
    flex: 1;
    overflow-y: auto;
    padding: 6px 0;
    scrollbar-width: thin;
    scrollbar-color: #2a2a4a transparent;
  }
  .playlist-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    font-size: 14px;
    cursor: default;
    color: #888;
    transition: background 0.15s;
  }
  .playlist-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .playlist-item.active {
    color: #e0e0e0;
    background: rgba(74, 222, 128, 0.08);
    border-left: 2px solid #4ade80;
    padding-left: 10px; /* compensate for border */
  }
  .item-icon {
    flex-shrink: 0;
    opacity: 0.7;
    display: flex;
  }
  .item-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .item-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }
  .item-dur {
    font-size: 12px;
    color: #666;
  }
  .item-loop {
    font-size: 12px;
    color: #4ade80;
    opacity: 0.8;
  }
  .now-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #4ade80;
    flex-shrink: 0;
    animation: pulse 1.4s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }
</style>
