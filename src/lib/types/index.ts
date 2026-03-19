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
  ttsAudioFolder: string;
  hasApiKey: boolean;
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
  ttsAudioFolder: '',
  hasApiKey: false,
};

export const DEFAULT_CONFIG: AppConfig = {
  settings: DEFAULT_SETTINGS,
  schedules: [],
};

export const ALL_DAYS = [1, 2, 3, 4, 5, 6, 7];

// ── ElevenLabs TTS Types ────────────────────────────────────────────────────

export type TTSStatus = 'pending' | 'generating' | 'completed' | 'failed' | 'deleted';

export interface TTSHistoryItem {
  id: string;
  historyItemId: string | null;
  text: string;
  voiceId: string;
  voiceName: string;
  modelId: string;
  language: string | null;
  stability: number | null;
  similarityBoost: number | null;
  speed: number | null;
  characterCount: number;
  localFilePath: string | null;
  status: TTSStatus;
  createdAt: string;
  syncedAt: string | null;
}

export interface ElevenLabsVoice {
  voice_id: string;
  name: string;
  category?: string;
  labels?: Record<string, string>;
  preview_url?: string;
}

export interface ElevenLabsLanguage {
  language_id: string;
  name: string;
}

export interface ElevenLabsModel {
  model_id: string;
  name: string;
  description?: string;
  can_be_finetuned: boolean;
  can_do_text_to_speech: boolean;
  languages?: ElevenLabsLanguage[];
  max_characters_request_free_user: number;
  max_characters_request_subscribed_user: number;
}

export interface UserSubscription {
  character_count: number;
  character_limit: number;
  tier: string;
  next_character_count_reset_unix?: number;
}

export interface TTSGenerateRequest {
  text: string;
  voiceId: string;
  voiceName: string;
  modelId: string;
  language?: string;
  stability?: number;
  similarityBoost?: number;
  speed?: number;
}

export interface ElevenLabsHistoryItem {
  history_item_id: string;
  request_id: string;
  voice_id: string;
  voice_name: string;
  model_id: string;
  text: string;
  date_unix: number;
  character_count_change_from: number;
  character_count_change_to: number;
  state: string;
  settings?: {
    stability?: number;
    similarity_boost?: number;
    style?: number;
    use_speaker_boost?: boolean;
  };
}
