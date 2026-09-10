// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Allow audio and video autoplay without requiring user interaction in WebView2
        let current_args = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
        let flag = "--autoplay-policy=no-user-gesture-required";
        if !current_args.contains(flag) {
            let new_args = if current_args.is_empty() {
                flag.to_string()
            } else {
                format!("{} {}", current_args, flag)
            };
            std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", new_args);
        }
    }

    siputar_lib::run()
}
