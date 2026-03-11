mod commands;
mod scheduler;
mod tray;
mod types;

use std::sync::{Arc, Mutex};
use scheduler::SchedulerState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scheduler_state = Arc::new(Mutex::new(SchedulerState::new()));

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
        .setup(move |app| {
            let handle = app.handle().clone();

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
                if window.label() == "main" {
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
            commands::get_app_version,
            commands::check_file_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

