use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    scheduler::SchedulerState,
    tray::update_tray_menu,
    types::SchedulerStatus,
    keychain,
    elevenlabs::{ElevenLabsClient, Voice, Model, UserSubscription},
};

type StateArg<'a> = tauri::State<'a, Arc<Mutex<SchedulerState>>>;
type ElevenLabsArg<'a> = tauri::State<'a, Arc<ElevenLabsClient>>;
type DbArg<'a> = tauri::State<'a, Arc<Mutex<rusqlite::Connection>>>;

// TTS History Item for frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TTSHistoryItem {
    pub id: String,
    pub history_item_id: Option<String>,
    pub text: String,
    pub voice_id: String,
    pub voice_name: String,
    pub model_id: String,
    pub language: Option<String>,
    pub stability: Option<f64>,
    pub similarity_boost: Option<f64>,
    pub speed: Option<f64>,
    pub character_count: i64,
    pub local_file_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub synced_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TTSGenerateRequest {
    pub text: String,
    pub voice_id: String,
    pub voice_name: String,
    pub model_id: String,
    pub language: Option<String>,
    pub stability: Option<f64>,
    pub similarity_boost: Option<f64>,
    pub speed: Option<f64>,
}

#[tauri::command]
pub fn get_scheduler_status(state: StateArg) -> String {
    match state.lock() {
        Ok(s) => match s.status {
            SchedulerStatus::Active => "active".into(),
            SchedulerStatus::Paused => "paused".into(),
        },
        Err(_) => "active".into(),
    }
}

#[tauri::command]
pub fn pause_all(app: AppHandle, state: StateArg) {
    if let Ok(mut s) = state.lock() {
        s.status = SchedulerStatus::Paused;
    }
    let _ = app.emit("tray:status-changed", serde_json::json!({ "status": "paused" }));
    update_tray_menu(&app, &SchedulerStatus::Paused);
}

#[tauri::command]
pub fn resume_all(app: AppHandle, state: StateArg) {
    if let Ok(mut s) = state.lock() {
        s.status = SchedulerStatus::Active;
    }
    let _ = app.emit("tray:status-changed", serde_json::json!({ "status": "active" }));
    update_tray_menu(&app, &SchedulerStatus::Active);
}

#[tauri::command]
pub fn update_schedules(schedules: Vec<crate::types::Schedule>, state: StateArg) {
    if let Ok(mut s) = state.lock() {
        s.schedules = schedules;
        s.notified.clear(); // reset triggered set when config changes
    }
}

#[tauri::command]
pub fn open_mini_player(app: AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("mini-player").is_none() {
        WebviewWindowBuilder::new(&app, "mini-player", WebviewUrl::App("/mini-player".into()))
            .title("Now Playing")
            .inner_size(380.0, 480.0)
            .resizable(false)
            .always_on_top(true)
            .decorations(false)
            .skip_taskbar(true)
            .build()?;
    } else if let Some(w) = app.get_webview_window("mini-player") {
        w.show()?;
    }
    Ok(())
}

#[tauri::command]
pub fn close_mini_player(app: AppHandle) {
    if let Some(w) = app.get_webview_window("mini-player") {
        let _ = w.hide();
    }
}

#[tauri::command]
pub fn check_file_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

// ── Keychain Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn save_api_key(key: String) -> Result<(), String> {
    println!("COMMAND: save_api_key called (key length: {})", key.len());
    let result = tokio::task::spawn_blocking(move || {
        keychain::save_api_key(&key)
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))?;
    
    match &result {
        Ok(_) => println!("COMMAND: save_api_key returned SUCCESS"),
        Err(e) => println!("COMMAND: save_api_key returned ERROR: {}", e),
    }
    result
}

#[tauri::command]
pub async fn get_api_key_masked() -> Result<Option<String>, String> {
    println!("COMMAND: get_api_key_masked called");
    let result = tokio::task::spawn_blocking(|| {
        keychain::get_api_key_masked()
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))?;
    
    match &result {
        Ok(Some(masked)) => println!("COMMAND: get_api_key_masked returned: {}", masked),
        Ok(None) => println!("COMMAND: get_api_key_masked returned: None"),
        Err(e) => println!("COMMAND: get_api_key_masked returned ERROR: {}", e),
    }
    result
}

#[tauri::command]
pub async fn has_api_key() -> Result<bool, String> {
    println!("COMMAND: has_api_key called");
    let result = tokio::task::spawn_blocking(|| {
        keychain::get_api_key().map(|k| k.is_some())
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))?;
    
    match &result {
        Ok(true) => println!("COMMAND: has_api_key returned: true"),
        Ok(false) => println!("COMMAND: has_api_key returned: false"),
        Err(e) => println!("COMMAND: has_api_key returned ERROR: {}", e),
    }
    result
}

#[tauri::command]
pub async fn delete_api_key() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        keychain::delete_api_key()
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))?
}

#[tauri::command]
pub async fn test_api_key(key: String, client: ElevenLabsArg<'_>) -> Result<bool, String> {
    client.test_api_key(&key).await
}

// ── ElevenLabs API Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_subscription(client: ElevenLabsArg<'_>) -> Result<UserSubscription, String> {
    client.get_subscription().await
}

#[tauri::command]
pub async fn get_voices(client: ElevenLabsArg<'_>) -> Result<Vec<Voice>, String> {
    client.get_voices().await
}

#[tauri::command]
pub async fn get_models(client: ElevenLabsArg<'_>) -> Result<Vec<Model>, String> {
    client.get_models().await
}

// ── TTS Database Commands ──────────────────────────────────────────────────────

#[tauri::command]
pub fn get_tts_history(db: DbArg<'_>) -> Result<Vec<TTSHistoryItem>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, history_item_id, text, voice_id, voice_name, model_id, language,
                stability, similarity_boost, speed, character_count, local_file_path,
                status, created_at, synced_at
         FROM tts_history
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let items = stmt.query_map([], |row| {
        Ok(TTSHistoryItem {
            id: row.get(0)?,
            history_item_id: row.get(1)?,
            text: row.get(2)?,
            voice_id: row.get(3)?,
            voice_name: row.get(4)?,
            model_id: row.get(5)?,
            language: row.get(6)?,
            stability: row.get(7)?,
            similarity_boost: row.get(8)?,
            speed: row.get(9)?,
            character_count: row.get(10)?,
            local_file_path: row.get(11)?,
            status: row.get(12)?,
            created_at: row.get(13)?,
            synced_at: row.get(14)?,
        })
    }).map_err(|e| e.to_string())?;

    items.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_speech(
    app: AppHandle,
    request: TTSGenerateRequest,
    client: ElevenLabsArg<'_>,
    db: DbArg<'_>,
) -> Result<TTSHistoryItem, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let char_count = request.text.chars().count() as i64;

    // Insert pending record
    let item = TTSHistoryItem {
        id: id.clone(),
        history_item_id: None,
        text: request.text.clone(),
        voice_id: request.voice_id.clone(),
        voice_name: request.voice_name.clone(),
        model_id: request.model_id.clone(),
        language: request.language.clone(),
        stability: request.stability,
        similarity_boost: request.similarity_boost,
        speed: request.speed,
        character_count: char_count,
        local_file_path: None,
        status: "generating".to_string(),
        created_at: created_at.clone(),
        synced_at: None,
    };

    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tts_history (id, text, voice_id, voice_name, model_id, language,
             stability, similarity_boost, speed, character_count, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                &id,
                &request.text,
                &request.voice_id,
                &request.voice_name,
                &request.model_id,
                &request.language,
                &request.stability,
                &request.similarity_boost,
                &request.speed,
                char_count,
                "generating",
                &created_at,
            ],
        ).map_err(|e| e.to_string())?;
    }

    // Spawn background task to generate speech
    let client = Arc::clone(&*client);
    let db = Arc::clone(&*db);
    let app_handle = app.clone();
    let req = request.clone();
    let item_id = id.clone();

    tokio::spawn(async move {
        let result = client.generate_speech(
            &req.voice_id,
            &req.text,
            &req.model_id,
            req.language.as_deref(),
            req.stability,
            req.similarity_boost,
            req.speed,
        ).await;

        match result {
            Ok(audio_data) => {
                // Get audio folder from config
                let audio_folder = get_audio_folder(&app_handle);
                let filename = format!("{}_{}.mp3", 
                    req.voice_name.replace(' ', "_"),
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                );
                let file_path = std::path::Path::new(&audio_folder).join(&filename);

                // Ensure directory exists
                if let Some(parent) = file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                // Save audio file
                if let Err(e) = std::fs::write(&file_path, &audio_data) {
                    eprintln!("Failed to save audio: {}", e);
                    update_tts_status(&db, &item_id, "failed", None);
                    return;
                }

                let path_str = file_path.to_string_lossy().to_string();
                update_tts_status(&db, &item_id, "completed", Some(&path_str));
            }
            Err(e) => {
                eprintln!("TTS generation failed: {}", e);
                update_tts_status(&db, &item_id, "failed", None);
            }
        }
    });

    Ok(item)
}

fn update_tts_status(db: &Arc<Mutex<rusqlite::Connection>>, id: &str, status: &str, file_path: Option<&str>) {
    if let Ok(conn) = db.lock() {
        let _ = conn.execute(
            "UPDATE tts_history SET status = ?1, local_file_path = ?2 WHERE id = ?3",
            rusqlite::params![status, file_path, id],
        );
    }
}

fn get_audio_folder(app: &AppHandle) -> String {
    // Try to get from app data dir, fallback to Documents
    app.path().app_data_dir()
        .map(|p| p.join("audio").to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            dirs::document_dir()
                .map(|p| p.join("PlaybackAnnouncer").join("audio").to_string_lossy().to_string())
                .unwrap_or_else(|| "./audio".to_string())
        })
}

#[tauri::command]
pub async fn download_history_audio(
    history_item_id: String,
    app: AppHandle,
    client: ElevenLabsArg<'_>,
    db: DbArg<'_>,
) -> Result<String, String> {
    let audio_data = client.download_audio(&history_item_id).await?;

    let audio_folder = get_audio_folder(&app);
    let filename = format!("downloaded_{}.mp3", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let file_path = std::path::Path::new(&audio_folder).join(&filename);

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::write(&file_path, &audio_data).map_err(|e| e.to_string())?;

    let path_str = file_path.to_string_lossy().to_string();

    // Update database
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tts_history SET local_file_path = ?1 WHERE history_item_id = ?2",
            rusqlite::params![&path_str, &history_item_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(path_str)
}

#[tauri::command]
pub async fn delete_tts_item(
    id: String,
    history_item_id: Option<String>,
    client: ElevenLabsArg<'_>,
    db: DbArg<'_>,
) -> Result<(), String> {
    // Delete local file if exists
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT local_file_path FROM tts_history WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let file_path: Option<String> = stmt.query_row([&id], |row| row.get(0)).ok();
        
        if let Some(path) = file_path {
            let _ = std::fs::remove_file(&path);
        }
    }

    // Delete from ElevenLabs if we have the history ID
    if let Some(hist_id) = history_item_id {
        let _ = client.delete_history_item(&hist_id).await;
    }

    // Delete from local DB
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM tts_history WHERE id = ?1", [&id])
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn sync_elevenlabs_history(
    client: ElevenLabsArg<'_>,
    db: DbArg<'_>,
) -> Result<(), String> {
    let history_response = client.get_history(Some(100), None).await?;

    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    for item in history_response.history {
        // Check if already exists
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM tts_history WHERE history_item_id = ?1",
            [&item.history_item_id],
            |row| row.get(0),
        ).unwrap_or(false);

        if !exists {
            let created_at = chrono::DateTime::from_timestamp(item.date_unix, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| now.clone());

            let char_count = item.character_count_change_to - item.character_count_change_from;

            conn.execute(
                "INSERT INTO tts_history (id, history_item_id, text, voice_id, voice_name, model_id,
                 stability, similarity_boost, character_count, status, created_at, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    &item.history_item_id,
                    &item.text,
                    &item.voice_id,
                    &item.voice_name,
                    &item.model_id,
                    item.settings.as_ref().and_then(|s| s.stability),
                    item.settings.as_ref().and_then(|s| s.similarity_boost),
                    char_count.abs(),
                    "completed",
                    &created_at,
                    &now,
                ],
            ).ok();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn open_audio_folder(file_path: String) -> Result<(), String> {
    let path = std::path::Path::new(&file_path);
    let folder = path.parent().unwrap_or(path);
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
