<script lang="ts">
  import "../../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { configStore } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import { IconCalendarEvent, IconMicrophone, IconPlayerPlay } from "@tabler/icons-svelte";

  let { children } = $props();

  const tr = $derived(t());
  const currentPath = $derived($page.url.pathname);

  async function openMiniPlayer() {
    await invoke("open_mini_player").catch(() => {});
  }

  // Apply theme to <html> element
  $effect(() => {
    const theme = configStore.settings.theme;
    const html = document.documentElement;
    if (theme === "auto") {
      const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      html.setAttribute("data-theme", dark ? "dark" : "light");
    } else {
      html.setAttribute("data-theme", theme);
    }
  });

  // Watch system theme changes when in auto mode
  onMount(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      if (configStore.settings.theme === "auto") {
        document.documentElement.setAttribute(
          "data-theme",
          e.matches ? "dark" : "light",
        );
      }
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });
</script>

<div class="app-shell">
  <header class="header">
    <div class="header-left">
      <div style="display: inline-flex; gap: .5rem; align-items: center;">
        <img src="/app-icon.png" alt="Playback Announcer" class="app-icon" />
        <h1 class="app-title">{tr.app.name}</h1>
      </div>
      <nav class="nav-tabs">
        <a
          href="/app/schedules"
          class="nav-tab"
          class:active={currentPath === "/app/schedules" || currentPath === "/"}
        >
          <IconCalendarEvent size={16} />
          <span>{tr.nav.schedules}</span>
        </a>
        <a
          href="/app/audio"
          class="nav-tab"
          class:active={currentPath === "/app/audio"}
        >
          <IconMicrophone size={16} />
          <span>{tr.nav.audio}</span>
        </a>
      </nav>
    </div>
    <div class="header-right">
      <button class="mini-player-btn" onclick={openMiniPlayer} title={tr.nav.miniPlayer}>
        <IconPlayerPlay size={16} />
        <span>{tr.nav.miniPlayer}</span>
      </button>
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
    justify-content: space-between;
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

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mini-player-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    cursor: pointer;
    transition: var(--transition);
  }

  .mini-player-btn:hover {
    color: var(--color-text);
    background: var(--color-surface-3);
    border-color: var(--color-border-hover);
  }

  .app-icon {
    width: 30px;
    height: 30px;
    flex-shrink: 0;
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
