use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

// These imports are used in conditional compilation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

use crate::{scheduler::SchedulerState, types::SchedulerStatus};

fn load_tray_icon(app: &AppHandle, status: &SchedulerStatus) -> Option<Image<'static>> {
    let icon_name = match status {
        SchedulerStatus::Active => "tray-active.png",
        SchedulerStatus::Paused => "tray-paused.png",
    };

    // Try resource dir first (production), then crate root (dev)
    let path = app
        .path()
        .resource_dir()
        .ok()
        .map(|d| d.join("icons").join(icon_name))
        .filter(|p| p.exists())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|d| d.join("icons").join(icon_name))
                .filter(|p| p.exists())
        });

    if let Some(p) = path {
        Image::from_path(p).ok()
    } else {
        None
    }
}

pub fn setup_tray(app: &AppHandle, state: Arc<Mutex<SchedulerState>>) -> tauri::Result<()> {
    let pause_item = MenuItem::with_id(app, "pause-resume", "Jeda Semua Jadwal", true, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Buka Aplikasi", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let exit_item = MenuItem::with_id(app, "exit", "Keluar", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&pause_item, &open_item, &sep, &exit_item])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Playback Announcer — Aktif");

    // Use the custom active icon if available, otherwise fall back to window icon
    if let Some(icon) = load_tray_icon(app, &SchedulerStatus::Active) {
        builder = builder.icon(icon);
    } else {
        builder = builder.icon(app.default_window_icon().unwrap().clone());
    }

    builder
        .on_menu_event({
            let state = state.clone();
            move |app, event| match event.id().as_ref() {
                "pause-resume" => {
                    let new_status = if let Ok(mut s) = state.lock() {
                        if s.status == SchedulerStatus::Active {
                            s.status = SchedulerStatus::Paused;
                            SchedulerStatus::Paused
                        } else {
                            s.status = SchedulerStatus::Active;
                            SchedulerStatus::Active
                        }
                    } else {
                        return;
                    };
                    let _ = app.emit("tray:status-changed", serde_json::json!({
                        "status": new_status,
                    }));
                    update_tray_menu(app, &new_status);
                }
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "exit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|_tray, _event| {
            // On Windows, left-click shows the menu by default (system behavior)
            // Our handler would conflict, causing menu to flash and disappear
            // So we only handle left-click on macOS/Linux for convenience
            #[cfg(not(target_os = "windows"))]
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = _event
            {
                let app = _tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_tray_menu(app: &AppHandle, status: &SchedulerStatus) {
    if let Some(tray) = app.tray_by_id("main") {
        let (label, tooltip) = match status {
            SchedulerStatus::Active => ("Jeda Semua Jadwal", "Playback Announcer — Aktif"),
            SchedulerStatus::Paused => ("Lanjutkan Semua Jadwal", "Playback Announcer — Dijeda"),
        };
        let _ = tray.set_tooltip(Some(tooltip));

        // Switch tray icon
        if let Some(icon) = load_tray_icon(app, status) {
            let _ = tray.set_icon(Some(icon));
        }

        if let Ok(pause_item) = MenuItem::with_id(app, "pause-resume", label, true, None::<&str>) {
            if let Ok(open_item) = MenuItem::with_id(app, "open", "Buka Aplikasi", true, None::<&str>) {
                if let Ok(sep) = PredefinedMenuItem::separator(app) {
                    if let Ok(exit_item) = MenuItem::with_id(app, "exit", "Keluar", true, None::<&str>) {
                        if let Ok(menu) = Menu::with_items(app, &[&pause_item, &open_item, &sep, &exit_item]) {
                            let _ = tray.set_menu(Some(menu));
                        }
                    }
                }
            }
        }
    }
}
