<script lang="ts">
  import { uiStore, closePanel } from "$lib/stores/ui.svelte.js";
  import { fade } from "svelte/transition";
  import {
    configStore,
    saveConfig,
    revertConfig,
  } from "$lib/stores/config.svelte.js";
  import { t } from "$lib/i18n/index.svelte.js";
  import ScheduleEditor from "./ScheduleEditor.svelte";
  import MediaEditor from "./MediaEditor.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import { IconX } from "@tabler/icons-svelte";

  const tr = $derived(t());
  const sel = $derived(uiStore.selection);
  const isOpen = $derived(uiStore.panelOpen);
  const isDirty = $derived(configStore.isDirty);

  async function handleSave() {
    await saveConfig();
    closePanel();
  }

  function handleRevert() {
    revertConfig();
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="backdrop"
    role="presentation"
    onclick={closePanel}
    in:fade={{ duration: 160 }}
    out:fade={{ duration: 120 }}
  ></div>
{/if}

<aside class="panel" class:open={isOpen}>
  <div class="panel-header">
    <button class="close-btn" onclick={closePanel} aria-label="Close panel"
      ><IconX size={16} /></button
    >
  </div>

  <div class="panel-body">
    {#if sel.type === "settings"}
      <SettingsPanel />
    {:else if sel.type === "schedule" && sel.scheduleId}
      <ScheduleEditor scheduleId={sel.scheduleId} />
    {:else if sel.type === "media" && sel.scheduleId && sel.mediaId}
      <MediaEditor scheduleId={sel.scheduleId} mediaId={sel.mediaId} />
    {/if}
  </div>

  {#if isDirty}
    <div class="panel-footer">
      <button class="btn btn-ghost" onclick={handleRevert}
        >{tr.actions.revert}</button
      >
      <button class="btn btn-primary" onclick={handleSave}
        >{tr.actions.save}</button
      >
    </div>
  {/if}
</aside>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 199;
    transition: all 0.2s ease;
    background: rgba(0, 0, 0, 0.4);
    opacity: 1;
  }

  .panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 300px;
    z-index: 200;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform 0.2s ease;
  }
  .panel.open {
    transform: translateX(0);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    font-size: 16px;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }
  .close-btn:hover {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px 16px;
  }

  .panel-footer {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    justify-content: flex-end;
  }
</style>
