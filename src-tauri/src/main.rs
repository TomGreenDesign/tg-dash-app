// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();

            // Cmd+Shift+R (mac) / Ctrl+Shift+R (win/linux) for hard refresh
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(true) = event {
                    let _ = window_clone.eval(
                        r#"
                        document.addEventListener('keydown', function(e) {
                            if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'R') {
                                e.preventDefault();
                                window.location.reload();
                            }
                        });
                        "#,
                    );
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
