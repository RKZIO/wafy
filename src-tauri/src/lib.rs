use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

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

pub fn run() {
    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![update_unread_count, show_notification])
        .setup(|app| {
            // Build Tray Menu
            let open_item = MenuItem::with_id(app, "open", "Open Wafy", true, None::<&str>)?;
            let reload_item = MenuItem::with_id(app, "reload", "Reload", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[&open_item, &reload_item, &quit_item])?;

            // Setup Tray Icon
            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("Wafy")
                .menu(&tray_menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "reload" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.eval("window.location.reload()");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Window Close-to-Tray event handler
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running Wafy Tauri application");
}
