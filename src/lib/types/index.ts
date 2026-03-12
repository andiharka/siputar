export interface NotificationRule {
  offsetMinutes: number;
}

export interface MediaItem {
  id: string;
  path: string;
  loopCount: number;
  volume: number; // 0.0 – 1.0
}

export interface Schedule {
  id: string;
  time: string; // "HH:MM:SS"
  activeDays: number[]; // 1=Mon … 7=Sun
  notifications: NotificationRule[];
  enabled: boolean;
  media: MediaItem[];
}

export interface AppSettings {
  theme: 'light' | 'dark' | 'auto';
  language: 'id' | 'en';
  runOnStartup: boolean;
}

export interface AppConfig {
  settings: AppSettings;
  schedules: Schedule[];
}

export type SchedulerStatus = 'active' | 'paused';

export interface PlaylistItem {
  name: string;
  type: 'video' | 'audio';
  path: string;
  loopCount: number;
}

export interface PlaybackState {
  status: 'idle' | 'playing' | 'paused';
  scheduleId: string | null;
  mediaIndex: number;
  currentLoop: number;
  mediaPath: string | null;
  mediaType: 'video' | 'audio' | null;
  playlist: PlaylistItem[];
  currentIndex: number;
}

export type SelectionType = 'schedule' | 'media' | 'settings' | null;

export interface Selection {
  type: SelectionType;
  scheduleId?: string;
  mediaId?: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: 'auto',
  language: 'id',
  runOnStartup: false,
};

export const DEFAULT_CONFIG: AppConfig = {
  settings: DEFAULT_SETTINGS,
  schedules: [],
};

export const ALL_DAYS = [1, 2, 3, 4, 5, 6, 7];
