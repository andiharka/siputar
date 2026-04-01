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

#[derive(Debug, Clone, Copy)]
enum SystemTheme {
    Light,
    Dark,
}

fn detect_system_theme() -> SystemTheme {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we'll use template images which auto-adapt
        // Return Light as default since template images handle the theme
        SystemTheme::Light
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::ptr;
        use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_CURRENT_USER};
        use winapi::um::winnt::{KEY_READ, REG_DWORD};
        use winapi::shared::winerror::ERROR_SUCCESS;
        
        unsafe {
            let mut hkey = ptr::null_mut();
            let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect::<Vec<u16>>();
            
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_READ,
                &mut hkey,
            ) == ERROR_SUCCESS as i32 {
                let value_name = "AppsUseLightTheme\0".encode_utf16().collect::<Vec<u16>>();
                let mut value: u32 = 0;
                let mut value_size: u32 = std::mem::size_of::<u32>() as u32;
                let mut value_type: u32 = REG_DWORD;
                
                let result = RegQueryValueExW(
                    hkey,
                    value_name.as_ptr(),
                    ptr::null_mut(),
                    &mut value_type,
                    &mut value as *mut u32 as *mut u8,
                    &mut value_size,
                );
                
                RegCloseKey(hkey);
                
                if result == ERROR_SUCCESS as i32 {
                    return if value == 0 { SystemTheme::Dark } else { SystemTheme::Light };
                }
            }
        }
        
        // Default to light theme if detection fails
        SystemTheme::Light
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try to detect GTK theme
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.to_lowercase().contains("dark") {
                return SystemTheme::Dark;
            }
        }
        
        // Try gsettings for GNOME
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if let Ok(theme_name) = String::from_utf8(output.stdout) {
                if theme_name.to_lowercase().contains("dark") {
                    return SystemTheme::Dark;
                }
            }
        }
        
        // Default to light theme
        SystemTheme::Light
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        SystemTheme::Light
    }
}

fn load_tray_icon(app: &AppHandle, status: &SchedulerStatus) -> Option<Image<'static>> {
    let base_name = match status {
        SchedulerStatus::Active => "tray-active",
        SchedulerStatus::Paused => "tray-paused",
    };

    // Determine which icon variant to use based on OS
    let icon_name = {
        #[cfg(target_os = "macos")]
        {
            // On macOS, use the template icons (black silhouettes on transparent)
            // The icon_as_template(true) flag tells macOS to auto-adapt colors
            format!("{}Template.png", base_name)
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // On Windows/Linux, use theme-specific icons
            let theme = detect_system_theme();
            let theme_suffix = match theme {
                SystemTheme::Light => "light",
                SystemTheme::Dark => "dark",
            };
            format!("{}-{}.png", base_name, theme_suffix)
        }
    };

    // Try resource dir first (production), then crate root (dev)
    let path = app
        .path()
        .resource_dir()
        .ok()
        .map(|d| d.join("icons").join(&icon_name))
        .filter(|p| p.exists())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|d| d.join("icons").join(&icon_name))
                .filter(|p| p.exists())
        })
        .or_else(|| {
            // Fallback to original icons if themed versions don't exist
            let fallback_name = format!("{}.png", base_name);
            app.path()
                .resource_dir()
                .ok()
                .map(|d| d.join("icons").join(&fallback_name))
                .filter(|p| p.exists())
                .or_else(|| {
                    std::env::current_dir()
                        .ok()
                        .map(|d| d.join("icons").join(&fallback_name))
                        .filter(|p| p.exists())
                })
        });

    if let Some(p) = path {
        Image::from_path(p).ok()
    } else {
        None
    }
}

pub fn setup_tray(app: &AppHandle, state: Arc<Mutex<SchedulerState>>) -> tauri::Result<()> {
    let pause_item = MenuItem::with_id(app, "pause-resume", "Jeda Semua Jadwal", true, None::<&str>)?;
    let mini_player_item = MenuItem::with_id(app, "show-mini-player", "Tampilkan Mini Player", true, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "Buka Aplikasi", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let exit_item = MenuItem::with_id(app, "exit", "Keluar", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&pause_item, &mini_player_item, &open_item, &sep, &exit_item])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Playback Announcer — Aktif");

    // Use the custom active icon if available, otherwise fall back to window icon
    if let Some(icon) = load_tray_icon(app, &SchedulerStatus::Active) {
        builder = builder.icon(icon);
    } else {
        builder = builder.icon(app.default_window_icon().unwrap().clone());
    }

    // On macOS, set icon as template so it auto-adapts to menubar theme (light/dark)
    #[cfg(target_os = "macos")]
    {
        builder = builder.icon_as_template(true);
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
                "show-mini-player" => {
                    if let Some(window) = app.get_webview_window("mini-player") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
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
            // On macOS, ensure icon remains a template for auto-adapting to theme
            #[cfg(target_os = "macos")]
            {
                let _ = tray.set_icon_as_template(true);
            }
        }

        if let Ok(pause_item) = MenuItem::with_id(app, "pause-resume", label, true, None::<&str>) {
            if let Ok(mini_player_item) = MenuItem::with_id(app, "show-mini-player", "Tampilkan Mini Player", true, None::<&str>) {
                if let Ok(open_item) = MenuItem::with_id(app, "open", "Buka Aplikasi", true, None::<&str>) {
                    if let Ok(sep) = PredefinedMenuItem::separator(app) {
                        if let Ok(exit_item) = MenuItem::with_id(app, "exit", "Keluar", true, None::<&str>) {
                            if let Ok(menu) = Menu::with_items(app, &[&pause_item, &mini_player_item, &open_item, &sep, &exit_item]) {
                                let _ = tray.set_menu(Some(menu));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn refresh_tray_icon(app: &AppHandle, status: &SchedulerStatus) {
    if let Some(tray) = app.tray_by_id("main") {
        // Update tray icon to reflect current system theme
        if let Some(icon) = load_tray_icon(app, status) {
            let _ = tray.set_icon(Some(icon));
            // On macOS, ensure icon remains a template for auto-adapting to theme
            #[cfg(target_os = "macos")]
            {
                let _ = tray.set_icon_as_template(true);
            }
        }
    }
}
