mod commands;
mod elevenlabs;
mod keychain;
mod scheduler;
mod tray;
mod types;

use std::sync::{Arc, Mutex};
use scheduler::SchedulerState;
use tauri::Manager;
use rusqlite::Connection;

fn init_database(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    
    let db_path = app_data_dir.join("tts_history.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tts_history (
            id TEXT PRIMARY KEY,
            history_item_id TEXT,
            text TEXT NOT NULL,
            voice_id TEXT NOT NULL,
            voice_name TEXT NOT NULL,
            model_id TEXT NOT NULL,
            language TEXT,
            stability REAL,
            similarity_boost REAL,
            speed REAL,
            character_count INTEGER NOT NULL DEFAULT 0,
            local_file_path TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL,
            synced_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tts_status ON tts_history(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tts_created ON tts_history(created_at DESC)",
        [],
    )?;

    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scheduler_state = Arc::new(Mutex::new(SchedulerState::new()));
    let elevenlabs_client = Arc::new(elevenlabs::ElevenLabsClient::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(scheduler_state.clone())
        .manage(elevenlabs_client)
        .setup(move |app| {
            let handle = app.handle().clone();

            // Initialize SQLite database
            let db = init_database(app)?;
            app.manage(Arc::new(Mutex::new(db)));

            // Setup system tray
            tray::setup_tray(&handle, scheduler_state.clone())?;

            // Start background scheduler thread
            scheduler::start_scheduler(handle, scheduler_state.clone());

            // Hide main window from taskbar on macOS when running in background
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing when user clicks X
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" || window.label() == "mini-player" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_scheduler_status,
            commands::pause_all,
            commands::resume_all,
            commands::update_schedules,
            commands::open_mini_player,
            commands::close_mini_player,
            commands::toggle_mini_player_devtools,
            commands::get_app_version,
            commands::check_file_exists,
            // Keychain commands
            commands::save_api_key,
            commands::get_api_key_masked,
            commands::has_api_key,
            commands::delete_api_key,
            commands::test_api_key,
            // ElevenLabs API commands
            commands::get_subscription,
            commands::get_voices,
            commands::get_models,
            // TTS commands
            commands::get_tts_history,
            commands::generate_speech,
            commands::download_history_audio,
            commands::delete_tts_item,
            commands::sync_elevenlabs_history,
            commands::open_audio_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}