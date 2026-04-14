import { LazyStore } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';
import { v4 as uuidv4 } from 'uuid';
import type { AppConfig, AppSettings, Schedule, MediaItem, NotificationRule } from '$lib/types/index.js';
import { DEFAULT_CONFIG, ALL_DAYS } from '$lib/types/index.js';
import { setLocale } from '$lib/i18n/index.svelte.js';

const STORE_KEY = 'config';

let _store: LazyStore | null = null;

function getStore(): LazyStore {
  if (!_store) {
    _store = new LazyStore('app-config.json', { defaults: {} });
  }
  return _store;
}

// ── Reactive State ─────────────────────────────────────────────────────────────

let _config = $state<AppConfig>(structuredClone(DEFAULT_CONFIG));
let _saved = $state<AppConfig>(structuredClone(DEFAULT_CONFIG));
let _loading = $state(true);

export const configStore = {
  get config() { return _config; },
  get schedules() { return _config.schedules; },
  get settings() { return _config.settings; },
  get loading() { return _loading; },
  get isDirty() {
    return JSON.stringify(_config) !== JSON.stringify(_saved);
  },
};

// ── Load / Save ────────────────────────────────────────────────────────────────

export async function loadConfig(): Promise<void> {
  try {
    const store = getStore();
    const saved = await store.get<AppConfig>(STORE_KEY);
    if (saved) {
      _config = structuredClone(saved);
      _saved = structuredClone(saved);
    } else {
      _config = structuredClone(DEFAULT_CONFIG);
      _saved = structuredClone(DEFAULT_CONFIG);
    }
    setLocale(_config.settings.language as 'id' | 'en');
    await syncSchedulerState();
    // Validate media file paths in background
    validateMediaPaths();
  } catch (e) {
    console.error('Failed to load config:', e);
  } finally {
    _loading = false;
  }
}

export async function saveConfig(): Promise<void> {
  try {
    const store = getStore();
    await store.set(STORE_KEY, $state.snapshot(_config));
    await store.save();
    _saved = structuredClone($state.snapshot(_config));
    await syncSchedulerState();
  } catch (e) {
    console.error('Failed to save config:', e);
    throw e;
  }
}

export function revertConfig(): void {
  _config = structuredClone($state.snapshot(_saved));
}

async function syncSchedulerState(): Promise<void> {
  try {
    await invoke('update_schedules', { schedules: $state.snapshot(_config.schedules) });
  } catch (_) { /* Rust not running in dev without tauri */ }
}

// ── Settings Mutations ─────────────────────────────────────────────────────────

export function updateSettings(patch: Partial<AppSettings>): void {
  _config.settings = { ..._config.settings, ...patch };
  if (patch.language) setLocale(patch.language as 'id' | 'en');
}

// ── Schedule Mutations ─────────────────────────────────────────────────────────

export function addSchedule(): Schedule {
  const schedule: Schedule = {
    id: uuidv4(),
    name: 'Jadwal Baru',
    time: '08:00:00',
    activeDays: [...ALL_DAYS],
    notifications: [{ offsetMinutes: 1 }],
    enabled: true,
    media: [],
  };
  _config.schedules = [..._config.schedules, schedule];
  invoke('log_schedule_create', { id: schedule.id, time: schedule.time }).catch(() => { });
  return schedule;
}

export function updateSchedule(id: string, patch: Partial<Omit<Schedule, 'id' | 'media'>>): void {
  _config.schedules = _config.schedules.map(s => s.id === id ? { ...s, ...patch } : s);
  invoke('log_schedule_update', { id, changes: Object.keys(patch).join(', ') }).catch(() => { });
}

export function deleteSchedule(id: string): void {
  _config.schedules = _config.schedules.filter(s => s.id !== id);
  invoke('log_schedule_delete', { id }).catch(() => { });
}

export function addNotification(scheduleId: string, rule: NotificationRule): void {
  _config.schedules = _config.schedules.map(s =>
    s.id === scheduleId ? { ...s, notifications: [...s.notifications, rule] } : s
  );
}

export function removeNotification(scheduleId: string, index: number): void {
  _config.schedules = _config.schedules.map(s =>
    s.id === scheduleId
      ? { ...s, notifications: s.notifications.filter((_, i) => i !== index) }
      : s
  );
}

export function updateNotification(scheduleId: string, index: number, rule: NotificationRule): void {
  _config.schedules = _config.schedules.map(s =>
    s.id === scheduleId
      ? { ...s, notifications: s.notifications.map((n, i) => i === index ? rule : n) }
      : s
  );
}

// ── Media Mutations ────────────────────────────────────────────────────────────

export function addMedia(scheduleId: string, paths: string[]): void {
  _config.schedules = _config.schedules.map(s => {
    if (s.id !== scheduleId) return s;
    const newItems: MediaItem[] = paths.map(path => ({
      id: uuidv4(),
      path,
      loopCount: 1,
      volume: 1.0,
    }));
    return { ...s, media: [...s.media, ...newItems] };
  });
}

export function updateMedia(scheduleId: string, mediaId: string, patch: Partial<Omit<MediaItem, 'id' | 'path'>>): void {
  _config.schedules = _config.schedules.map(s =>
    s.id === scheduleId
      ? { ...s, media: s.media.map(m => m.id === mediaId ? { ...m, ...patch } : m) }
      : s
  );
}

export function deleteMedia(scheduleId: string, mediaId: string): void {
  _config.schedules = _config.schedules.map(s =>
    s.id === scheduleId ? { ...s, media: s.media.filter(m => m.id !== mediaId) } : s
  );
}

export function reorderMedia(scheduleId: string, fromIndex: number, toIndex: number): void {
  _config.schedules = _config.schedules.map(s => {
    if (s.id !== scheduleId) return s;
    const media = [...s.media];
    const [item] = media.splice(fromIndex, 1);
    media.splice(toIndex, 0, item);
    return { ...s, media };
  });
}

export function moveMediaToSchedule(fromScheduleId: string, fromIndex: number, toScheduleId: string, toIndex?: number): void {
  let movedItem: MediaItem | undefined;
  _config.schedules = _config.schedules.map(s => {
    if (s.id === fromScheduleId) {
      const media = [...s.media];
      [movedItem] = media.splice(fromIndex, 1);
      return { ...s, media };
    }
    return s;
  });
  if (!movedItem) return;
  const item = movedItem;
  _config.schedules = _config.schedules.map(s => {
    if (s.id === toScheduleId) {
      const media = [...s.media];
      const insertAt = toIndex !== undefined && toIndex <= media.length ? toIndex : media.length;
      media.splice(insertAt, 0, { ...item, id: uuidv4() });
      return { ...s, media };
    }
    return s;
  });
}

// ── File Validation ────────────────────────────────────────────────────────────

let _missingFiles = $state(new Set<string>());

export const validationStore = {
  get missingFiles() { return _missingFiles; },
  isMissing: (path: string) => _missingFiles.has(path),
};

async function validateMediaPaths(): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const missing = new Set<string>();
    for (const schedule of _config.schedules) {
      for (const media of schedule.media) {
        const ok = await invoke<boolean>('check_file_exists', { path: media.path }).catch(() => true);
        if (!ok) missing.add(media.path);
      }
    }
    _missingFiles = missing;
  } catch { /* not available in dev */ }
}
