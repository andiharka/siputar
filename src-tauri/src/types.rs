use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRule {
    pub offset_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub path: String,
    pub loop_count: u32,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub time: String, // "HH:MM:SS"
    pub active_days: Vec<u8>, // 1=Mon..7=Sun
    pub notifications: Vec<NotificationRule>,
    pub enabled: bool,
    pub media: Vec<MediaItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AppSettings {
    pub theme: String,   // "light" | "dark" | "auto"
    pub language: String, // "id" | "en"
    pub run_on_startup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            language: "id".into(),
            run_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AppConfig {
    pub settings: AppSettings,
    pub schedules: Vec<Schedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SchedulerStatus {
    Active,
    Paused,
}
