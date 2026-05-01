import { invoke } from '@tauri-apps/api/core';
import { closeTTSPanel as closePanel } from '$lib/stores/ui.svelte.js';
import { configStore } from '$lib/stores/config.svelte.js';
import type {
  TTSHistoryItem,
  ElevenLabsVoice,
  ElevenLabsModel,
  UserSubscription,
  TTSGenerateRequest,
} from '$lib/types/index.js';

// ── Reactive State ─────────────────────────────────────────────────────────────

let _history = $state<TTSHistoryItem[]>([]);
let _voices = $state<ElevenLabsVoice[]>([]);
let _models = $state<ElevenLabsModel[]>([]);
let _subscription = $state<UserSubscription | null>(null);
let _isLoading = $state(false);
let _isSyncing = $state(false);
let _isOnline = $state(typeof navigator !== 'undefined' ? navigator.onLine : true);
let _generatingId = $state<string | null>(null);
let _playingId = $state<string | null>(null);
let _error = $state<string | null>(null);

// Cache flags to prevent duplicate API requests
let _voicesCached = false;
let _modelsCached = false;

export const ttsStore = {
  get history() { return _history; },
  get voices() { return _voices; },
  get models() { return _models; },
  get subscription() { return _subscription; },
  get isLoading() { return _isLoading; },
  get isSyncing() { return _isSyncing; },
  get isOnline() { return _isOnline; },
  set isOnline(value: boolean) { _isOnline = value; },
  get generatingId() { return _generatingId; },
  get playingId() { return _playingId; },
  set playingId(value: string | null) { _playingId = value; },
  get error() { return _error; },
};

export function clearError(): void {
  _error = null;
}

// ── Load from SQLite ───────────────────────────────────────────────────────────

export async function loadTTSHistory(): Promise<void> {
  _isLoading = true;
  _error = null;
  try {
    const history = await invoke<TTSHistoryItem[]>('get_tts_history');
    _history = history;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('Failed to load TTS history:', msg);
    _error = `Failed to load history: ${msg}`;
  } finally {
    _isLoading = false;
  }
}

// ── ElevenLabs API Calls ───────────────────────────────────────────────────────

export async function loadSubscription(): Promise<void> {
  _error = null;
  try {
    const sub = await invoke<UserSubscription>('get_subscription');
    _subscription = sub;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('Failed to load subscription:', msg);
    _error = `Failed to load subscription: ${msg}`;
  }
}

export async function loadVoices(forceRefresh = false): Promise<void> {
  if (_voicesCached && !forceRefresh && _voices.length > 0) return;
  _error = null;
  try {
    const voices = await invoke<ElevenLabsVoice[]>('get_voices');
    _voices = voices;
    _voicesCached = true;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('Failed to load voices:', msg);
    _error = `Failed to load voices: ${msg}`;
  }
}

export async function loadModels(forceRefresh = false): Promise<void> {
  if (_modelsCached && !forceRefresh && _models.length > 0) return;
  _error = null;
  try {
    const models = await invoke<ElevenLabsModel[]>('get_models');
    _models = models.filter(m => m.can_do_text_to_speech);
    _modelsCached = true;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('Failed to load models:', msg);
    _error = `Failed to load models: ${msg}`;
  }
}

export async function syncWithElevenLabs(): Promise<void> {
  if (_isSyncing) return;
  _isSyncing = true;
  _error = null;
  try {
    await Promise.all([
      loadSubscription(),
      invoke('sync_elevenlabs_history'),
    ]);
    await loadTTSHistory();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('Failed to sync with ElevenLabs:', msg);
    _error = `Failed to sync: ${msg}`;
  } finally {
    _isSyncing = false;
  }
}

// ── Generate Speech ────────────────────────────────────────────────────────────

export async function generateSpeech(request: TTSGenerateRequest): Promise<void> {
  closePanel();

  try {
    const item = await invoke<TTSHistoryItem>('generate_speech', { 
      request,
      audioFolder: configStore.settings.ttsAudioFolder
    });
    _generatingId = item.id;
    _history = [item, ..._history];

    // Poll for completion or listen for events
    await waitForGeneration(item.id);
  } catch (e) {
    console.error('Failed to generate speech:', e);
    throw e;
  } finally {
    _generatingId = null;
  }
}

async function waitForGeneration(id: string): Promise<void> {
  // Poll the status until completed or failed
  const maxAttempts = 60; // 60 seconds max
  for (let i = 0; i < maxAttempts; i++) {
    await new Promise(resolve => setTimeout(resolve, 1000));

    try {
      const history = await invoke<TTSHistoryItem[]>('get_tts_history');
      const item = history.find(h => h.id === id);

      if (item) {
        _history = history;
        if (item.status === 'completed' || item.status === 'failed') {
          // Refresh subscription to update credits
          await loadSubscription();
          return;
        }
      }
    } catch (e) {
      console.error('Error polling generation status:', e);
    }
  }
}

// ── Audio Actions ──────────────────────────────────────────────────────────────

export async function downloadAudio(historyItemId: string): Promise<void> {
  try {
    const filePath = await invoke<string>('download_history_audio', { 
      historyItemId,
      audioFolder: configStore.settings.ttsAudioFolder 
    });
    await loadTTSHistory();
    return;
  } catch (e) {
    console.error('Failed to download audio:', e);
    throw e;
  }
}

export async function deleteHistoryItem(id: string, historyItemId: string | null): Promise<void> {
  try {
    await invoke('delete_tts_item', { id, historyItemId });
    _history = _history.filter(h => h.id !== id);
  } catch (e) {
    console.error('Failed to delete history item:', e);
    throw e;
  }
}

export async function openAudioFolder(filePath: string): Promise<void> {
  try {
    await invoke('open_audio_folder', { filePath });
  } catch (e) {
    console.error('Failed to open folder:', e);
  }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

export function getHistoryItem(id: string): TTSHistoryItem | undefined {
  return _history.find(h => h.id === id);
}

export function formatCredits(sub: UserSubscription | null): string {
  if (!sub) return '—';
  const remaining = sub.character_limit - sub.character_count;
  return `${remaining.toLocaleString()} / ${sub.character_limit.toLocaleString()}`;
}

export function getCreditsPercent(sub: UserSubscription | null): number {
  if (!sub || sub.character_limit === 0) return 0;
  return ((sub.character_limit - sub.character_count) / sub.character_limit) * 100;
}
