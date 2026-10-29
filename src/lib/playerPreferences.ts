import { LazyStore } from '@tauri-apps/plugin-store';
import { emitTo, listen } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import { setLocale } from '$lib/i18n/index.svelte.js';
import type { AppConfig, AppSettings } from '$lib/types/index.js';
import { DEFAULT_SETTINGS } from '$lib/types/index.js';

export type PlayerPreferences = Pick<AppSettings, 'theme' | 'language'>;

export function publishPlayerPreferences(preferences: PlayerPreferences): void {
  void emitTo('mini-player', 'app:preferences', preferences).catch(() => {});
}

// Read display preferences only. Loading the full config would re-arm the scheduler
// in this second webview and could interfere with unsaved edits in the main window.
export function connectPlayerPreferences(): () => void {
  let preferences: PlayerPreferences = {
    theme: DEFAULT_SETTINGS.theme,
    language: DEFAULT_SETTINGS.language,
  };
  let revision = 0;
  let disposed = false;
  let unlisten: (() => void) | undefined;
  const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');

  function apply(value: PlayerPreferences) {
    if (disposed) return;
    preferences = value;
    setLocale(value.language);
    document.documentElement.dataset.theme = value.theme === 'auto'
      ? systemTheme.matches ? 'dark' : 'light'
      : value.theme;
  }

  const updateSystemTheme = () => apply(preferences);
  apply(preferences);
  systemTheme.addEventListener('change', updateSystemTheme);

  if (isTauri()) {
    void (async () => {
      try {
        const stop = await listen<PlayerPreferences>('app:preferences', ({ payload }) => {
          if (disposed) return;
          revision++;
          apply(payload);
        });
        if (disposed) { stop(); return; }
        unlisten = stop;
        void emitTo('main', 'mini-player:preferences-request').catch(() => {});
        const store = new LazyStore('app-config.json', { defaults: {} });
        const saved = await store.get<AppConfig>('config');
        // A live response from the main window takes precedence over disk.
        if (saved && revision === 0) apply(saved.settings);
      } catch (error) {
        console.warn('[MiniPlayer] Could not read display preferences:', error);
      }
    })();
  }

  return () => {
    disposed = true;
    unlisten?.();
    systemTheme.removeEventListener('change', updateSystemTheme);
  };
}
