import type { Selection } from '$lib/types/index.js';

let _selection = $state<Selection>({ type: null });
let _panelOpen = $state(false);
let _ttsPanelOpen = $state(false);
let _confirmDialog = $state<{
  open: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
  onCancel?: () => void;
}>({ open: false, title: '', message: '', onConfirm: () => {} });

export const uiStore = {
  get selection() { return _selection; },
  get panelOpen() { return _panelOpen; },
  get ttsPanelOpen() { return _ttsPanelOpen; },
  get confirmDialog() { return _confirmDialog; },
};

export function openPanel(selection: Selection): void {
  _selection = selection;
  _panelOpen = true;
}

export function closePanel(): void {
  _panelOpen = false;
  _selection = { type: null };
}

export function openSettings(): void {
  _selection = { type: 'settings' };
  _panelOpen = true;
}

export function openTTSPanel(): void {
  _ttsPanelOpen = true;
}

export function closeTTSPanel(): void {
  _ttsPanelOpen = false;
}

export function showConfirm(title: string, message: string, onConfirm: () => void, onCancel?: () => void): void {
  _confirmDialog = { open: true, title, message, onConfirm, onCancel };
}

export function closeConfirm(): void {
  _confirmDialog = { open: false, title: '', message: '', onConfirm: () => {} };
}
