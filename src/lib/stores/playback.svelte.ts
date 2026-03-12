import type { PlaybackState, SchedulerStatus } from '$lib/types/index.js';

let _playback = $state<PlaybackState>({
  status: 'idle',
  scheduleId: null,
  mediaIndex: 0,
  currentLoop: 0,
  mediaPath: null,
  mediaType: null,
  playlist: [],
  currentIndex: 0,
});

let _schedulerStatus = $state<SchedulerStatus>('active');

export const playbackStore = {
  get state() { return _playback; },
  get schedulerStatus() { return _schedulerStatus; },
  get isPlaying() { return _playback.status === 'playing'; },
  get isIdle() { return _playback.status === 'idle'; },
};

export function setPlaybackState(patch: Partial<PlaybackState>): void {
  _playback = { ..._playback, ...patch };
}

export function setSchedulerStatus(status: SchedulerStatus): void {
  _schedulerStatus = status;
}
