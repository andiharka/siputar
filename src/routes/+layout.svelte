<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { configStore } from '$lib/stores/config.svelte.js';
  import { loadConfig } from '$lib/stores/config.svelte.js';
  import { t } from '$lib/i18n/index.svelte.js';
  import { IconCalendarEvent, IconMicrophone } from '@tabler/icons-svelte';

  let { children } = $props();

  const tr = $derived(t());
  const currentPath = $derived($page.url.pathname);

  // Apply theme to <html> element
  $effect(() => {
    const theme = configStore.settings.theme;
    const html = document.documentElement;
    if (theme === 'auto') {
      const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      html.setAttribute('data-theme', dark ? 'dark' : 'light');
    } else {
      html.setAttribute('data-theme', theme);
    }
  });

  // Watch system theme changes when in auto mode
  onMount(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => {
      if (configStore.settings.theme === 'auto') {
        document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light');
      }
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  });
</script>

<div class="app-shell">
  <header class="header">
    <div class="header-left">
      <h1 class="app-title">{tr.app.name}</h1>
      <nav class="nav-tabs">
        <a
          href="/schedules"
          class="nav-tab"
          class:active={currentPath === '/schedules' || currentPath === '/'}
        >
          <IconCalendarEvent size={16} />
          <span>{tr.nav.schedules}</span>
        </a>
        <a
          href="/audio"
          class="nav-tab"
          class:active={currentPath === '/audio'}
        >
          <IconMicrophone size={16} />
          <span>{tr.nav.audio}</span>
        </a>
      </nav>
    </div>
  </header>

  <main class="main-content">
    {@render children()}
  </main>

  <footer class="footer">
    &copy; 2026 Andi &ndash; Disperpusip Jawa Timur. All rights reserved.
  </footer>
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
    padding: 0 20px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    height: 52px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 24px;
    height: 100%;
  }

  .app-title {
    font-size: 16px;
    font-weight: 700;
    white-space: nowrap;
  }

  .nav-tabs {
    display: flex;
    gap: 4px;
    height: 100%;
  }

  .nav-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 16px;
    height: 100%;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    text-decoration: none;
    border-bottom: 2px solid transparent;
    transition: var(--transition);
  }

  .nav-tab:hover {
    color: var(--color-text);
    background: var(--color-surface-2);
  }

  .nav-tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
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
