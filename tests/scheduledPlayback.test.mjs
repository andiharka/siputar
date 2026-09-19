import { beforeAll, beforeEach, expect, mock, test } from 'bun:test';

const listeners = new Map();
const sent = [];
const commands = [];
let openFailure = false;
let readyReply = true;
let loadCount = 0;
let state = {};
const savedSchedule = {
  id: 'morning', name: 'Morning', time: '08:00:00', enabled: true,
  activeDays: [1, 2, 3, 4, 5], notifications: [], loopCount: 1,
  media: [
    { path: 'C:\\Announcements\\Bell.wav', volume: 0.8, loopCount: 2 },
    { path: 'D:\\Audio\\Welcome.mp3', volume: 1, loopCount: 1 },
  ],
};
const config = {
  savedSchedules: [savedSchedule],
  schedules: [savedSchedule],
  isDirty: false,
};
async function fire(name, payload = {}) {
  for (const callback of [...(listeners.get(name) ?? [])]) await callback({ payload });
}
async function flush() {
  for (let i = 0; i < 30; i++) await Promise.resolve();
}
mock.module('@tauri-apps/api/event', () => ({
  listen: async (name, callback) => {
    if (!listeners.has(name)) listeners.set(name, new Set());
    listeners.get(name).add(callback);
    return () => listeners.get(name).delete(callback);
  },
  emit: async (name, payload) => {
    sent.push({ name, payload });
    await fire(name, payload);
    if (name === 'mini-player:ready-request' && readyReply) {
      await fire('mini-player:ready', { requestId: payload.requestId });
    }
  },
}));
mock.module('@tauri-apps/api/core', () => ({
  invoke: async (command) => {
    commands.push(command);
    if (command === 'open_mini_player' && openFailure) throw new Error('Window creation failed');
    if (command === 'get_scheduler_status') return 'active';
  },
}));
mock.module('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ listen: async () => () => {}, hide: async () => {} }),
}));
mock.module('../src/lib/stores/config.svelte.ts', () => ({
  configStore: config,
  loadConfig: async () => {
    loadCount++;
    // Simulate Rust firing as soon as schedules are armed during startup.
    expect(listeners.get('playback:ended').size).toBe(1);
    await fire('scheduler:play', { scheduleId: savedSchedule.id });
  },
  saveConfig: async () => {}, revertConfig: () => {},
}));
mock.module('../src/lib/stores/playback.svelte.ts', () => ({
  setPlaybackState: (patch) => { state = { ...state, ...patch }; },
  setSchedulerStatus: () => {},
}));
mock.module('../src/lib/stores/ui.svelte.ts', () => ({ showConfirm: () => {} }));
mock.module('../src/lib/i18n/index.svelte.ts', () => ({ t: () => ({}) }));
mock.module('../src/lib/utils/thumbnail.ts', () => ({
  getMediaType: () => 'audio', getFileName: (path) => path.split(/[\\/]/).pop(),
}));
const { initializeScheduledPlayback, startScheduledPlayback } = await import('../src/lib/scheduledPlayback.ts');
const starts = () => sent.filter((event) => event.name === 'playback:start').map((event) => event.payload);

beforeAll(async () => {
  await initializeScheduledPlayback();
  await flush();
  expect(starts()).toHaveLength(1);
});
beforeEach(async () => {
  await fire('playback:stop');
  sent.length = 0;
  commands.length = 0;
  openFailure = false;
  readyReply = true;
  config.schedules = [savedSchedule];
});

test('initialization stays singular across page navigation and scheduled events remain connected', async () => {
  await initializeScheduledPlayback();
  await initializeScheduledPlayback();
  expect(loadCount).toBe(1);
  expect(listeners.get('scheduler:play').size).toBe(1);
  await fire('scheduler:play', { scheduleId: 'morning' });
  await flush();
  expect(starts()).toHaveLength(1);
  expect(starts()[0].path).toBe(savedSchedule.media[0].path);
  expect(listeners.get('mini-player:ready').size).toBe(0);
});

test('scheduled playback uses saved media; manual preview uses unsaved edits', async () => {
  config.schedules = [{ ...savedSchedule, media: [{ path: 'C:\\Edited.wav', volume: 1, loopCount: 1 }] }];
  await fire('scheduler:play', { scheduleId: 'morning' });
  await flush();
  expect(starts()[0].path).toBe(savedSchedule.media[0].path);
  await startScheduledPlayback('morning');
  expect(starts()[1].path).toBe('C:\\Edited.wav');
});

test('queue repeats tracks, ignores stale end events, then advances without reopening the window', async () => {
  await startScheduledPlayback('morning');
  const first = starts()[0];
  await fire('playback:ended', first);
  await flush();
  expect(starts()[1].path).toBe(first.path);
  await fire('playback:ended', first);
  await flush();
  expect(starts()).toHaveLength(2);
  await fire('playback:ended', starts()[1]);
  await flush();
  expect(starts()[2].path).toBe(savedSchedule.media[1].path);
  expect(commands.filter((name) => name === 'open_mini_player')).toHaveLength(1);
});

test('a failed window open cleans up and a later schedule can start', async () => {
  openFailure = true;
  await startScheduledPlayback('morning');
  expect(state.status).toBe('idle');
  expect(starts()).toHaveLength(0);
  openFailure = false;
  await startScheduledPlayback('morning');
  expect(starts()).toHaveLength(1);
});

test('stop while waiting for readiness prevents delayed playback', async () => {
  readyReply = false;
  const pending = startScheduledPlayback('morning');
  await flush();
  const request = sent.find((event) => event.name === 'mini-player:ready-request');
  await fire('playback:stop');
  await fire('mini-player:ready', request.payload);
  await pending;
  expect(starts()).toHaveLength(0);
  expect(state.status).toBe('idle');
});

test('overlapping starts let the newest session win', async () => {
  await Promise.all([startScheduledPlayback('morning'), startScheduledPlayback('morning')]);
  expect(starts()).toHaveLength(1);
});

test('a WebView2 ready response after three seconds still starts playback', async () => {
  readyReply = false;
  const pending = startScheduledPlayback('morning');
  await flush();
  const request = sent.find((event) => event.name === 'mini-player:ready-request');
  await new Promise((resolve) => setTimeout(resolve, 3100));
  await fire('mini-player:ready', request.payload);
  await pending;
  expect(starts()).toHaveLength(1);
});

test('playback success is recorded only after media actually starts', async () => {
  await startScheduledPlayback('morning');
  expect(commands.filter((name) => name === 'report_playback')).toHaveLength(0);
  await fire('playback:started', starts()[0]);
  expect(commands.filter((name) => name === 'report_playback')).toHaveLength(1);
});

test('reset restarts the playlist from the first track', async () => {
  await startScheduledPlayback('morning');
  await fire('playback:reset');
  await new Promise((resolve) => setTimeout(resolve, 550));
  expect(starts()).toHaveLength(2);
  expect(starts()[1].path).toBe(savedSchedule.media[0].path);
  expect(starts()[1].sessionId).not.toBe(starts()[0].sessionId);
});

test('stop cancels readiness even when the mini-player never replies', async () => {
  readyReply = false;
  const pending = startScheduledPlayback('morning');
  await flush();
  await fire('playback:stop');
  await pending;
  expect(starts()).toHaveLength(0);
  expect(listeners.get('mini-player:ready').size).toBe(0);
}, 1000);
