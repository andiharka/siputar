<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  onMount(() => {
    goto('/schedules', { replaceState: true });
  });
</script>

<div class="loading">Loading...</div>

<style>
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: var(--color-text-muted);
  }
</style>
