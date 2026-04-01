use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

const MAX_LOG_SIZE: u64 = 1_000_000; // 1MB
const MAX_ROTATED_FILES: usize = 3;

static LOG_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub ts: String,
    pub cat: String,
    pub act: String,
    pub data: serde_json::Value,
}

pub fn init(app_data_dir: PathBuf) {
    let logs_dir = app_data_dir.join("logs");
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        eprintln!("[ActivityLog] Failed to create logs directory: {}", e);
        return;
    }
    
    if let Ok(mut dir) = LOG_DIR.lock() {
        *dir = Some(logs_dir);
    }
}

fn get_log_path() -> Option<PathBuf> {
    LOG_DIR.lock().ok()?.clone().map(|d| d.join("activity.log"))
}

fn rotate_if_needed(log_path: &PathBuf) {
    let metadata = match fs::metadata(log_path) {
        Ok(m) => m,
        Err(_) => return,
    };
    
    if metadata.len() < MAX_LOG_SIZE {
        return;
    }
    
    // Rotate: activity.log.3 -> delete, activity.log.2 -> .3, etc.
    for i in (1..=MAX_ROTATED_FILES).rev() {
        let from = if i == 1 {
            log_path.clone()
        } else {
            log_path.with_extension(format!("log.{}", i - 1))
        };
        let to = log_path.with_extension(format!("log.{}", i));
        
        if i == MAX_ROTATED_FILES {
            let _ = fs::remove_file(&to);
        }
        
        if from.exists() {
            let _ = fs::rename(&from, &to);
        }
    }
}

pub fn log_event(category: &str, action: &str, data: serde_json::Value) {
    let log_path = match get_log_path() {
        Some(p) => p,
        None => {
            eprintln!("[ActivityLog] Log directory not initialized");
            return;
        }
    };
    
    rotate_if_needed(&log_path);
    
    let entry = ActivityLogEntry {
        ts: Utc::now().to_rfc3339(),
        cat: category.to_string(),
        act: action.to_string(),
        data,
    };
    
    let line = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ActivityLog] Failed to serialize entry: {}", e);
            return;
        }
    };
    
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[ActivityLog] Failed to open log file: {}", e);
            return;
        }
    };
    
    if let Err(e) = writeln!(file, "{}", line) {
        eprintln!("[ActivityLog] Failed to write to log file: {}", e);
    }
}

pub fn get_logs(limit: usize, offset: usize) -> Vec<ActivityLogEntry> {
    let log_path = match get_log_path() {
        Some(p) => p,
        None => return Vec::new(),
    };
    
    let mut all_entries: Vec<ActivityLogEntry> = Vec::new();
    
    // Read from rotated files first (oldest), then main log (newest)
    let mut files_to_read: Vec<PathBuf> = Vec::new();
    for i in (1..=MAX_ROTATED_FILES).rev() {
        let rotated = log_path.with_extension(format!("log.{}", i));
        if rotated.exists() {
            files_to_read.push(rotated);
        }
    }
    if log_path.exists() {
        files_to_read.push(log_path);
    }
    
    for file_path in files_to_read {
        if let Ok(file) = File::open(&file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Ok(entry) = serde_json::from_str::<ActivityLogEntry>(&line) {
                    all_entries.push(entry);
                }
            }
        }
    }
    
    // Reverse to get newest first
    all_entries.reverse();
    
    // Apply pagination
    all_entries.into_iter().skip(offset).take(limit).collect()
}

pub fn clear_logs() -> Result<(), String> {
    let log_path = match get_log_path() {
        Some(p) => p,
        None => return Err("Log directory not initialized".to_string()),
    };
    
    // Remove main log file
    if log_path.exists() {
        fs::remove_file(&log_path).map_err(|e| e.to_string())?;
    }
    
    // Remove rotated files
    for i in 1..=MAX_ROTATED_FILES {
        let rotated = log_path.with_extension(format!("log.{}", i));
        if rotated.exists() {
            let _ = fs::remove_file(&rotated);
        }
    }
    
    Ok(())
}

pub fn get_log_file_path() -> Option<String> {
    get_log_path().map(|p| p.to_string_lossy().to_string())
}

pub fn export_logs(destination: PathBuf) -> Result<(), String> {
    let _log_path = match get_log_path() {
        Some(p) => p,
        None => return Err("Log directory not initialized".to_string()),
    };
    
    // Collect all entries from all log files
    let entries = get_logs(usize::MAX, 0);
    
    // Write as JSON array to destination
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize logs: {}", e))?;
    
    fs::write(&destination, json)
        .map_err(|e| format!("Failed to write export file: {}", e))?;
    
    Ok(())
}

// Convenience functions for logging specific events
pub fn log_schedule_create(id: &str, time: &str) {
    log_event("schedule", "create", serde_json::json!({
        "id": id,
        "time": time
    }));
}

pub fn log_schedule_update(id: &str, changes: &str) {
    log_event("schedule", "update", serde_json::json!({
        "id": id,
        "changes": changes
    }));
}

pub fn log_schedule_delete(id: &str) {
    log_event("schedule", "delete", serde_json::json!({
        "id": id
    }));
}

pub fn log_voice_generate(id: &str, text_preview: &str, voice_name: &str) {
    let preview = if text_preview.len() > 50 {
        format!("{}...", &text_preview[..50])
    } else {
        text_preview.to_string()
    };
    
    log_event("voice", "generate", serde_json::json!({
        "id": id,
        "text": preview,
        "voice": voice_name
    }));
}

pub fn log_voice_delete(id: &str) {
    log_event("voice", "delete", serde_json::json!({
        "id": id
    }));
}

pub fn log_voice_play(id: &str, file_path: &str) {
    log_event("voice", "play", serde_json::json!({
        "id": id,
        "file": file_path
    }));
}

pub fn log_playback_running(schedule_id: Option<&str>) {
    log_event("playback", "running", serde_json::json!({
        "scheduleId": schedule_id
    }));
}

pub fn log_playback_paused() {
    log_event("playback", "paused", serde_json::json!({}));
}

pub fn log_playback_stopped() {
    log_event("playback", "stopped", serde_json::json!({}));
}
