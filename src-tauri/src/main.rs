// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    webview::{PageLoadEvent, PageLoadPayload},
    Manager, Webview, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_updater::UpdaterExt;

const APP_URL: &str = "https://dash.tomgreen.uk";
const APP_HOST: &str = "dash.tomgreen.uk";
const OTHER_HOST: &str = "docs.tomgreen.uk";
const OTHER_SCHEME: &str = "tg-docs";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // --- Build menu ---
            let hard_refresh = MenuItemBuilder::with_id("hard_refresh", "Hard Refresh")
                .accelerator("CmdOrCtrl+Shift+R")
                .build(app)?;
            let return_home = MenuItemBuilder::with_id("return_home", "Return to Dash")
                .build(app)?;
            let sign_out =
                MenuItemBuilder::with_id("sign_out", "Sign Out").build(app)?;
            let clear_data =
                MenuItemBuilder::with_id("clear_data", "Clear Local Data").build(app)?;
            let version_item = MenuItemBuilder::with_id(
                "version",
                &format!("Version {}", app.package_info().version),
            )
            .enabled(false)
            .build(app)?;

            let app_submenu = SubmenuBuilder::new(app, "App")
                .item(&hard_refresh)
                .item(&return_home)
                .separator()
                .item(&sign_out)
                .item(&clear_data)
                .separator()
                .item(&version_item)
                .quit()
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let window_submenu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .close_window()
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&edit_submenu)
                .item(&window_submenu)
                .build()?;

            app.set_menu(menu)?;

            // --- Create window with navigation handler ---
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("TG Dash")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .on_navigation(move |url| {
                    let url_str = url.as_str();

                    // Catch marker URLs from our window.open override
                    if url_str.starts_with("__tg_external__:") {
                        let real_url = &url_str["__tg_external__:".len()..];
                        let _ = Command::new("open").arg(real_url).spawn();
                        return false;
                    }

                    // Cross-app links → open via deep link scheme
                    if url.host_str() == Some(OTHER_HOST) {
                        let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                        let deep = format!("{}://{}{}", OTHER_SCHEME, url.path(), query);
                        let _ = Command::new("open").arg(&deep).spawn();
                        return false;
                    }

                    // Same-domain → allow
                    if url.host_str() == Some(APP_HOST) {
                        return true;
                    }

                    // External https/http → open in browser
                    if url.scheme() == "https" || url.scheme() == "http" {
                        let _ = Command::new("open").arg(url_str).spawn();
                        return false;
                    }

                    true // allow tauri://, about:blank, etc.
                })
                .build()?;

            // --- Handle menu events ---
            let menu_window = window.clone();
            app.on_menu_event(move |_app_handle, event| {
                match event.id().as_ref() {
                    "clear_data" | "sign_out" => {
                        let _ = menu_window.clear_all_browsing_data();
                        let _ = menu_window.eval(&format!(
                            "window.location.replace('{}')",
                            APP_URL
                        ));
                    }
                    "hard_refresh" => {
                        let _ = menu_window.eval("window.location.reload()");
                    }
                    "return_home" => {
                        let _ = menu_window.eval(&format!(
                            "window.location.replace('{}')",
                            APP_URL
                        ));
                    }
                    _ => {}
                }
            });

            // --- Handle incoming deep links ---
            let dl_window = window.clone();
            app.deep_link().on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
                    let path = url.path();
                    let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                    let target = format!("{}{}{}", APP_URL, path, query);
                    let _ = dl_window.eval(&format!(
                        "window.location.replace('{}')",
                        target
                    ));
                }
            });

            // --- Check for updates in the background ---
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let updater = match app_handle.updater() {
                    Ok(u) => u,
                    Err(_) => return,
                };
                if let Ok(Some(update)) = updater.check().await {
                    let _ = update
                        .download_and_install(|_chunk, _total| {}, || {})
                        .await;
                    app_handle.restart();
                }
            });

            Ok(())
        })
        .on_page_load(|webview: &Webview, payload: &PageLoadPayload<'_>| {
            if payload.event() == PageLoadEvent::Finished {
                let _ = webview.eval(
                    r#"
                    (function() {
                        if (window.__tg_patched) return;
                        window.__tg_patched = true;

                        // Override window.open — WKWebView swallows these silently.
                        // Convert to a marker navigation that Rust intercepts.
                        window.open = function(url) {
                            if (!url) return null;
                            try {
                                const parsed = new URL(url, window.location.origin);
                                const f = document.createElement('iframe');
                                f.style.display = 'none';
                                f.src = '__tg_external__:' + parsed.href;
                                document.body.appendChild(f);
                                setTimeout(() => f.remove(), 100);
                            } catch(_) {}
                            return null;
                        };

                        // Intercept <a target="_blank"> clicks — convert to marker navigations
                        document.addEventListener('click', function(e) {
                            const link = e.target.closest('a');
                            if (!link || !link.href) return;
                            if (link.target === '_blank') {
                                e.preventDefault();
                                try {
                                    const url = new URL(link.href);
                                    const f = document.createElement('iframe');
                                    f.style.display = 'none';
                                    f.src = '__tg_external__:' + url.href;
                                    document.body.appendChild(f);
                                    setTimeout(() => f.remove(), 100);
                                } catch(_) {}
                            }
                        }, true);

                        // Fix drag-to-reorder showing copy/insert instead of move
                        document.addEventListener('dragstart', function(e) {
                            if (e.target.closest('.tiptap') || e.target.closest('.drag-handle-icon')) {
                                e.dataTransfer.effectAllowed = 'move';
                            }
                        }, true);
                        document.addEventListener('dragover', function(e) {
                            if (e.target.closest('.tiptap')) {
                                e.dataTransfer.dropEffect = 'move';
                            }
                        }, true);
                    })();
                    "#,
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
