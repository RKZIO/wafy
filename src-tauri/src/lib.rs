use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuBuilder, MenuItem, PredefinedMenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    webview::{PermissionKind, PermissionResponse},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

// Permission state shared between the menu handler and the permission callback
#[derive(Clone, PartialEq)]
enum MicPermission {
    NotAsked,
    Allowed,
    Denied,
}

#[tauri::command]
fn update_unread_count(count: u32, app: AppHandle) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tooltip = if count > 0 {
            format!("Wafy ({} unread)", count)
        } else {
            "Wafy".to_string()
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    if let Some(window) = app.get_webview_window("main") {
        let title = if count > 0 {
            format!("({}) Wafy", count)
        } else {
            "Wafy".to_string()
        };
        let _ = window.set_title(&title);
    }

    Ok(())
}

#[tauri::command]
fn show_notification(title: String, body: String, app: AppHandle) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
    Ok(())
}

/// Show a native zenity dialog to ask the user about microphone access.
/// Returns true if user clicked Allow.
fn ask_microphone_permission() -> bool {
    // Capture DISPLAY now (called from WebKit thread which may lose it)
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());

    std::process::Command::new("zenity")
        .args([
            "--question",
            "--title=Microphone Access",
            "--text=Allow Wafy to use your microphone for voice messages?",
            "--ok-label=Allow",
            "--cancel-label=Deny",
            "--width=360",
        ])
        .env("DISPLAY", &display)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn run(single_instance_listener: Option<std::os::unix::net::UnixListener>) {
    // Shared permission state: starts as NotAsked
    let mic_state: Arc<Mutex<MicPermission>> = Arc::new(Mutex::new(MicPermission::NotAsked));

    let mic_state_for_handler = mic_state.clone();
    let mic_state_for_reset = mic_state.clone();

    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![update_unread_count, show_notification])
        // Global permission handler for microphone / camera requests from the webview
        .on_permission_request(move |_webview, kind| match kind {
            PermissionKind::Microphone | PermissionKind::Camera => {
                let state = mic_state_for_handler.lock().unwrap();
                match *state {
                    MicPermission::Allowed => PermissionResponse::Allow,
                    MicPermission::Denied => PermissionResponse::Deny,
                    MicPermission::NotAsked => {
                        // Ask user via native dialog (this runs on a background thread)
                        drop(state); // release lock before blocking call
                        let allowed = ask_microphone_permission();
                        let mut state = mic_state_for_handler.lock().unwrap();
                        *state = if allowed {
                            MicPermission::Allowed
                        } else {
                            MicPermission::Denied
                        };
                        if allowed {
                            PermissionResponse::Allow
                        } else {
                            PermissionResponse::Deny
                        }
                    }
                }
            }
            PermissionKind::Notifications => PermissionResponse::Allow,
            _ => PermissionResponse::Default,
        })
        .setup(move |app| {
            // ── Top Menu Bar ──────────────────────────────────────────────
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&MenuItem::with_id(app, "reload", "Reload", true, Some("F5"))?)
                .separator()
                .item(&MenuItem::with_id(app, "close-tray", "Close to Tray", true, None::<&str>)?)
                .item(&MenuItem::with_id(app, "quit", "Quit", true, Some("Ctrl+Q"))?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::undo(app, None)?)
                .item(&PredefinedMenuItem::redo(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&MenuItem::with_id(app, "zoom-in", "Zoom In", true, Some("Ctrl+="))?)
                .item(&MenuItem::with_id(app, "zoom-out", "Zoom Out", true, Some("Ctrl+-"))?)
                .item(&MenuItem::with_id(app, "zoom-reset", "Reset Zoom", true, Some("Ctrl+0"))?)
                .separator()
                .item(&MenuItem::with_id(app, "fullscreen", "Toggle Full Screen", true, Some("F11"))?)
                .build()?;

            let prefs_menu = SubmenuBuilder::new(app, "Preferences")
                .item(&MenuItem::with_id(app, "mic-allow", "Allow Microphone Access", true, None::<&str>)?)
                .item(&MenuItem::with_id(app, "mic-deny", "Deny Microphone Access", true, None::<&str>)?)
                .item(&MenuItem::with_id(app, "mic-reset", "Reset Microphone Permission", true, None::<&str>)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&prefs_menu)
                .build()?;

            // Set app-wide menu and retain a handle for explicit window attachment
            let menu_for_window = menu.clone();
            let _ = app.set_menu(menu);

            // ── Menu Event Handler ─────────────────────────────────────────
            let mic_reset_clone = mic_state_for_reset.clone();
            app.on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "reload" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("window.location.reload()");
                        }
                    }
                    "close-tray" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    "zoom-in" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("document.body.style.zoom = (parseFloat(document.body.style.zoom || 1) + 0.1).toFixed(1)");
                        }
                    }
                    "zoom-out" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("document.body.style.zoom = Math.max(0.5, (parseFloat(document.body.style.zoom || 1) - 0.1)).toFixed(1)");
                        }
                    }
                    "zoom-reset" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("document.body.style.zoom = '1'");
                        }
                    }
                    "fullscreen" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let is_full = w.is_fullscreen().unwrap_or(false);
                            let _ = w.set_fullscreen(!is_full);
                        }
                    }
                    "mic-allow" => {
                        let mut state = mic_reset_clone.lock().unwrap();
                        *state = MicPermission::Allowed;
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("window.location.reload()");
                        }
                    }
                    "mic-deny" => {
                        let mut state = mic_reset_clone.lock().unwrap();
                        *state = MicPermission::Denied;
                    }
                    "mic-reset" => {
                        let mut state = mic_reset_clone.lock().unwrap();
                        *state = MicPermission::NotAsked;
                    }
                    _ => {}
                }
            });

            // ── System Tray Icon ───────────────────────────────────────────
            let open_item = MenuItem::with_id(app, "tray-open", "Open Wafy", true, None::<&str>)?;
            let reload_item = MenuItem::with_id(app, "tray-reload", "Reload", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "tray-quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&open_item, &reload_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("Wafy")
                .menu(&tray_menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tray-open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "tray-reload" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.eval("window.location.reload()");
                        }
                    }
                    "tray-quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Badge/Notification init script ─────────────────────────────
            let badge_js = include_str!("../../src/badge_observer.js");
            let style_css = include_str!("../../src/style.css");
            let init_script = format!(
                r#"
                {}
                window.addEventListener('DOMContentLoaded', () => {{
                    try {{
                        const style = document.createElement('style');
                        style.textContent = `{}`;
                        document.head.appendChild(style);
                    }} catch (e) {{}}
                }});
                "#,
                badge_js,
                style_css.replace('`', "\\`")
            );

            // ── Main WebviewWindow ─────────────────────────────────────────
            let window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()),
            )
            .title("Wafy")
            .inner_size(1050.0, 720.0)
            .min_inner_size(600.0, 500.0)
            .resizable(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
            .initialization_script(&init_script)
            .build()?;

            // Explicitly attach menu to window (required on Linux — app.set_menu
            // only covers windows that exist at that moment)
            let _ = window.set_menu(menu_for_window);

            // Close → minimise to tray
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            // ── Single-instance: listen for activate signals ───────────────
            if let Some(listener) = single_instance_listener {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    use std::io::BufRead;
                    for stream in listener.incoming() {
                        if let Ok(stream) = stream {
                            let mut reader = std::io::BufReader::new(stream);
                            let mut line = String::new();
                            if reader.read_line(&mut line).is_ok() && line.trim() == "activate" {
                                if let Some(w) = app_handle.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.unminimize();
                                    let _ = w.set_focus();
                                }
                            }
                        }
                    }
                });
            }

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running Wafy Tauri application");
}
