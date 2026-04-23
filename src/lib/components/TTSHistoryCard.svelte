<script lang="ts">
  import { t } from "$lib/i18n/index.svelte.js";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    ttsStore,
    downloadAudio,
    deleteHistoryItem,
    openAudioFolder,
  } from "$lib/stores/tts.svelte.js";
  import { showConfirm } from "$lib/stores/ui.svelte.js";
  import type { TTSHistoryItem } from "$lib/types/index.js";
  import {
    IconPlayerPlay,
    IconPlayerStop,
    IconDownload,
    IconFolder,
    IconTrash,
    IconLoader,
    IconCopy,
  } from "@tabler/icons-svelte";

  interface Props {
    item: TTSHistoryItem;
  }

  let { item }: Props = $props();

  const tr = $derived(t());
  const playingId = $derived(ttsStore.playingId);
  const isPlaying = $derived(playingId === item.id);
  const hasLocalFile = $derived(!!item.localFilePath);
  const isGenerating = $derived(
    item.status === "generating" || item.status === "pending",
  );

  let isDownloading = $state(false);
  let audioElement: HTMLAudioElement | null = null;

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleString("id-ID", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function truncateText(text: string, maxLength: number = 100): string {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength) + "...";
  }

  function getStatusBadgeClass(status: string): string {
    switch (status) {
      case "completed":
        return "badge-success";
      case "failed":
        return "badge-danger";
      case "generating":
      case "pending":
        return "badge-warning";
      default:
        return "";
    }
  }

  function getStatusText(status: string): string {
    switch (status) {
      case "completed":
        return tr.tts.completed;
      case "failed":
        return tr.tts.failed;
      case "generating":
      case "pending":
        return tr.tts.generating;
      default:
        return status;
    }
  }

  async function handlePlay() {
    if (!item.localFilePath) return;

    if (isPlaying) {
      audioElement?.pause();
      ttsStore.playingId = null;
      return;
    }

    // Stop any currently playing audio
    if (ttsStore.playingId) {
      const event = new CustomEvent("tts-stop-audio");
      window.dispatchEvent(event);
    }

    audioElement = new Audio(convertFileSrc(item.localFilePath));
    audioElement.onended = () => {
      ttsStore.playingId = null;
    };
    audioElement.onerror = () => {
      ttsStore.playingId = null;
    };

    ttsStore.playingId = item.id;
    await audioElement.play();
  }

  async function handleDownload() {
    if (!item.historyItemId) return;
    isDownloading = true;
    try {
      await downloadAudio(item.historyItemId);
    } catch (e) {
      console.error("Download failed:", e);
    } finally {
      isDownloading = false;
    }
  }

  function handleOpenFolder() {
    if (item.localFilePath) {
      openAudioFolder(item.localFilePath);
    }
  }

  function handleDelete() {
    showConfirm(tr.actions.delete, tr.tts.deleteConfirm, async () => {
      await deleteHistoryItem(item.id, item.historyItemId);
    });
  }

  function handleCopyText() {
    navigator.clipboard.writeText(item.text);
    // show a toast notification for successful copy
    const event = new CustomEvent("show-toast", {
      detail: { message: "tr.tts.copiedToClipboard" },
    });
    window.dispatchEvent(event);
  }

  // Listen for stop audio event from other cards
  $effect(() => {
    const handler = () => {
      if (audioElement) {
        audioElement.pause();
        audioElement = null;
      }
    };
    window.addEventListener("tts-stop-audio", handler);
    return () => window.removeEventListener("tts-stop-audio", handler);
  });
</script>

<div class="card" class:generating={isGenerating}>
  <div class="card-content">
    <div class="text-preview-wrap">
      {#if item.name}
        <div class="item-name">{item.name}</div>
      {/if}
      <div class="text-preview">{truncateText(item.text)}</div>
    </div>
    <div class="meta">
      <span class="date">{formatDate(item.createdAt)}</span>
      <span class="voice">{item.voiceName}</span>
      <span class="badge {getStatusBadgeClass(item.status)}">
        {#if isGenerating}<IconLoader size={12} class="spinning" />{/if}
        {getStatusText(item.status)}
      </span>
    </div>
  </div>

  <div class="card-tooltip">
    <div class="tooltip-row">
      <div class="">{item.text}</div>
      <button
        class="btn btn-ghost"
        style="display: flex;flex-direction: column; justify-content: center;"
        onclick={handleCopyText}
      >
        <IconCopy size={16} />Copy
      </button>
    </div>
    <div
      style="max-width: 200px; border-left: 2px solid var(--color-border); padding-left: .5rem; margin-top: .125rem;"
    >
      <div class="tooltip-row">
        <span class="tooltip-label">Model:</span>
        <span class="tooltip-value">{item.modelId}</span>
      </div>
      {#if item.stability !== null}
        <div class="tooltip-row">
          <span class="tooltip-label">{tr.tts.stability}:</span>
          <span class="tooltip-value">{(item.stability * 100).toFixed(0)}%</span
          >
        </div>
      {/if}
      {#if item.similarityBoost !== null}
        <div class="tooltip-row">
          <span class="tooltip-label">{tr.tts.similarity}:</span>
          <span class="tooltip-value"
            >{(item.similarityBoost * 100).toFixed(0)}%</span
          >
        </div>
      {/if}
      <div class="tooltip-row">
        <span class="tooltip-label">{tr.tts.charLimit}:</span>
        <span class="tooltip-value">{item.characterCount.toLocaleString()}</span
        >
      </div>
    </div>
  </div>

  <div class="card-actions">
    {#if hasLocalFile}
      <button
        class="btn btn-icon"
        class:btn-primary={isPlaying}
        class:btn-ghost={!isPlaying}
        onclick={handlePlay}
        title={isPlaying ? tr.actions.stop : tr.actions.play}
      >
        {#if isPlaying}
          <IconPlayerStop size={16} />
        {:else}
          <IconPlayerPlay size={16} />
        {/if}
      </button>
      <button
        class="btn btn-ghost btn-icon"
        onclick={handleOpenFolder}
        title={tr.actions.openFolder}
      >
        <IconFolder size={16} />
      </button>
    {:else if item.historyItemId && !isGenerating}
      <button
        class="btn btn-ghost btn-icon"
        onclick={handleDownload}
        disabled={isDownloading}
        title={tr.actions.download}
      >
        {#if isDownloading}
          <IconLoader size={16} class="spinning" />
        {:else}
          <IconDownload size={16} />
        {/if}
      </button>
    {/if}

    {#if !isGenerating}
      <button
        class="btn btn-ghost btn-icon btn-danger-hover"
        onclick={handleDelete}
        title={tr.actions.delete}
      >
        <IconTrash size={16} />
      </button>
    {/if}
  </div>
</div>

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    position: relative;
    transition: var(--transition);
  }

  .card:hover {
    border-color: var(--color-primary);
  }

  .card.generating {
    opacity: 0.7;
  }

  .card-content {
    flex: 1;
    min-width: 0;
  }

  /* Wrapper that shows the full-text tooltip on hover */
  .text-preview-wrap {
    position: relative;
  }

  .text-full-tooltip {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    z-index: 200;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    box-shadow: var(--shadow-md);
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text);
    white-space: pre-wrap;
    word-break: break-word;
    max-width: 420px;
    max-height: 200px;
    overflow-y: auto;
    /* pointer-events: none; */
    /* animation */
    opacity: 0;
    visibility: hidden;
    transform: translateY(4px);
    transition:
      opacity 0.15s ease,
      visibility 0.15s ease,
      transform 0.15s ease;
  }
  .text-full-tooltip:hover {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .text-preview {
    font-size: 14px;
    line-height: 1.5;
    margin-bottom: 8px;
    color: var(--color-text);
  }

  .item-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 4px;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .date {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .voice {
    font-size: 12px;
    /* font-weight: 500; */
    color: var(--color-text-muted);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 100px;
    font-size: 11px;
    font-weight: 600;
  }

  .badge-success {
    background: #dcfce7;
    color: #166534;
  }

  .badge-danger {
    background: #fee2e2;
    color: #991b1b;
  }

  .badge-warning {
    background: #fef3c7;
    color: #92400e;
  }

  .card-tooltip {
    position: absolute;
    top: 100%;
    left: 16px;
    z-index: 100;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    box-shadow: var(--shadow-md);
    opacity: 0;
    visibility: hidden;
    transition:
      opacity 0.15s ease,
      visibility 0.15s ease;
    /* pointer-events: none; */
    min-width: 180px;
    max-width: 550px;
  }

  .card:hover .card-tooltip {
    opacity: 1;
    visibility: visible;
  }

  .card-tooltip:hover {
    opacity: 1;
    visibility: visible;
  }

  .tooltip-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 11px;
    padding: 2px 0;
  }

  .tooltip-label {
    color: var(--color-text-muted);
  }

  .tooltip-value {
    color: var(--color-text);
    font-weight: 500;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .btn-icon {
    padding: 6px;
    width: 32px;
    height: 32px;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  :global([data-theme="dark"]) .badge-success {
    background: #166534;
    color: #dcfce7;
  }

  :global([data-theme="dark"]) .badge-danger {
    background: #991b1b;
    color: #fee2e2;
  }

  :global([data-theme="dark"]) .badge-warning {
    background: #92400e;
    color: #fef3c7;
  }
</style>
