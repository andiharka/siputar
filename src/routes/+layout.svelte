<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { configStore } from '$lib/stores/config.svelte.js';
  import { loadConfig } from '$lib/stores/config.svelte.js';

  let { children } = $props();

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

{@render children()}
