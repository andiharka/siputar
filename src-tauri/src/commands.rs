use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{
    scheduler::SchedulerState,
    tray::update_tray_menu,
    types::SchedulerStatus,
};

type StateArg<'a> = tauri::State<'a, Arc<Mutex<SchedulerState>>>;

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
