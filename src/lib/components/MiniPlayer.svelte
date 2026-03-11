<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, emit } from '@tauri-apps/api/event';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { playbackStore, setPlaybackState } from '$lib/stores/playback.svelte.js';
  import { getFileName } from '$lib/utils/thumbnail.js';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { IconPlayerPause, IconPlayerPlay, IconPlayerStop } from '@tabler/icons-svelte';

  let videoEl = $state<HTMLVideoElement | null>(null);
  let audioEl = $state<HTMLAudioElement | null>(null);
  // Track which element is currently loaded/playing
  let activeType = $state<'video' | 'audio' | null>(null);

  let unlisteners: UnlistenFn[] = [];

  const pb = $derived(playbackStore.state);
  const fileName = $derived(pb.mediaPath ? getFileName(pb.mediaPath) : '');

  function activeEl(): HTMLVideoElement | HTMLAudioElement | null {
    if (activeType === 'video') return videoEl;
    if (activeType === 'audio') return audioEl;
    return null;
  }

  onMount(async () => {
    unlisteners.push(
      await listen<{ path: string; type: 'video' | 'audio'; volume: number }>('playback:start', ({ payload }) => {
        activeType = payload.type;
        setPlaybackState({ status: 'playing', mediaPath: payload.path, mediaType: payload.type });
        const el = payload.type === 'video' ? videoEl : audioEl;
        if (el) {
          el.src = convertFileSrc(payload.path);
          el.volume = payload.volume;
          el.play().catch(console.error);
        }
      }),
      await listen('playback:pause', () => {
        setPlaybackState({ status: 'paused' });
        activeEl()?.pause();
      }),
      await listen('playback:resume', () => {
        setPlaybackState({ status: 'playing' });
        activeEl()?.play().catch(() => {});
      }),
      await listen('playback:stop', () => {
        setPlaybackState({ status: 'idle', mediaPath: null, mediaType: null });
        activeType = null;
        if (videoEl) { videoEl.src = ''; videoEl.load(); }
        if (audioEl) { audioEl.src = ''; audioEl.load(); }
      }),
    );
  });

  onDestroy(() => {
    unlisteners.forEach(u => u());
  });

  function handleEnded() {
    emit('playback:ended', {});
  }

  async function handlePause() {
    await emit('playback:pause', {});
  }

  async function handleResume() {
    await emit('playback:resume', {});
  }

  async function handleStop() {
    await emit('playback:stop', {});
  }
</script>

<div class="player" data-tauri-drag-region>
  <div class="media-area" data-tauri-drag-region>
    <video
      bind:this={videoEl}
      class:hidden={pb.mediaType !== 'video'}
      onended={handleEnded}
      style="width:100%;height:100%;object-fit:contain;"
    ></video>
    <audio bind:this={audioEl} onended={handleEnded}></audio>
  </div>

  <div class="controls">
    <div class="info" title={pb.mediaPath ?? ''}>
      <span class="status-dot" class:playing={pb.status === 'playing'}></span>
      <span class="filename">{fileName || '—'}</span>
    </div>
    <div class="buttons">
      {#if pb.status === 'playing'}
        <button class="ctrl-btn" onclick={handlePause} title="Pause"><IconPlayerPause size={14} /></button>
      {:else}
        <button class="ctrl-btn" onclick={handleResume} title="Resume" disabled={pb.status === 'idle'}><IconPlayerPlay size={14} /></button>
      {/if}
      <button class="ctrl-btn" onclick={handleStop} title="Stop" disabled={pb.status === 'idle'}><IconPlayerStop size={14} /></button>
    </div>
  </div>
</div>

<style>
  :global(body) {
    background: #1a1a2e; color: #e0e0e0;
    margin: 0; font-family: -apple-system, BlinkMacSystemFont, sans-serif;
    overflow: hidden; height: 100vh;
  }

  .player {
    width: 100vw; height: 100vh;
    display: flex; flex-direction: column;
    background: #1a1a2e;
  }

  .media-area { flex: 1; overflow: hidden; background: #000; }
  .hidden { display: none; }

  .controls {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 12px; gap: 10px; flex-shrink: 0;
    background: #12122a;
  }

  .info { display: flex; align-items: center; gap: 6px; overflow: hidden; flex: 1; }
  .status-dot {
    width: 6px; height: 6px; border-radius: 50%;
    background: #555; flex-shrink: 0; transition: 0.2s;
  }
  .status-dot.playing { background: #4ade80; }
  .filename { font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .buttons { display: flex; gap: 6px; flex-shrink: 0; }
  .ctrl-btn {
    background: rgba(255,255,255,0.1); border: none; color: #e0e0e0;
    border-radius: 4px; padding: 4px 8px; cursor: pointer; font-size: 14px;
    transition: 0.15s;
  }
  .ctrl-btn:hover:not(:disabled) { background: rgba(255,255,255,0.2); }
  .ctrl-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
