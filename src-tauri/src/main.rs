// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    webview::{PageLoadEvent, PageLoadPayload},
    Manager, Webview,
};

const APP_URL: &str = "https://dash.tomgreen.uk";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // --- Build menu ---
            let hard_refresh = MenuItemBuilder::with_id("hard_refresh", "Hard Refresh")
                .accelerator("CmdOrCtrl+Shift+R")
                .build(app)?;
            let sign_out =
                MenuItemBuilder::with_id("sign_out", "Sign Out").build(app)?;
            let clear_data =
                MenuItemBuilder::with_id("clear_data", "Clear Local Data").build(app)?;

            let app_submenu = SubmenuBuilder::new(app, "App")
                .item(&hard_refresh)
                .separator()
                .item(&sign_out)
                .item(&clear_data)
                .separator()
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

            // --- Handle menu events ---
            let window = app.get_webview_window("main").unwrap();
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
                    _ => {}
                }
            });

            Ok(())
        })
        .on_page_load(|webview: &Webview, payload: &PageLoadPayload<'_>| {
            if payload.event() == PageLoadEvent::Finished {
                let _ = webview.eval(
                    r#"
                    (function() {
                        if (window.__tg_link_rewriter_installed) return;
                        window.__tg_link_rewriter_installed = true;

                        const SCHEME_MAP = {
                            'docs.tomgreen.uk': 'tg-docs',
                            'dash.tomgreen.uk': 'tg-dash'
                        };
                        const currentHost = window.location.hostname;

                        document.addEventListener('click', function(e) {
                            const link = e.target.closest('a');
                            if (!link) return;
                            try {
                                const url = new URL(link.href);
                                const scheme = SCHEME_MAP[url.hostname];
                                if (scheme && url.hostname !== currentHost) {
                                    e.preventDefault();
                                    const deepLink = scheme + '://' + url.pathname + url.search + url.hash;
                                    if (window.__TAURI__ && window.__TAURI__.shell) {
                                        window.__TAURI__.shell.open(deepLink);
                                    }
                                }
                            } catch(_) {}
                        }, true);
                    })();
                    "#,
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
