// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_else(|_| "default".to_string());
    std::env::temp_dir().join(format!("wafy-instance-{}.sock", user))
}

fn check_single_instance() -> Option<UnixListener> {
    let socket_path = get_socket_path();

    match UnixListener::bind(&socket_path) {
        Ok(listener) => Some(listener),
        Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => {
            // Check if another instance is actually responding
            if let Ok(mut stream) = UnixStream::connect(&socket_path) {
                let _ = stream.write_all(b"activate\n");
                // Already running: signal existing instance and exit immediately
                std::process::exit(0);
            } else {
                // Stale socket from previous crash, clean up and rebind
                let _ = std::fs::remove_file(&socket_path);
                UnixListener::bind(&socket_path).ok()
            }
        }
        Err(_) => None,
    }
}

fn main() {
    // Single-instance enforcement: guarantees only ONE instance ever runs
    let single_instance_listener = check_single_instance();

    // NVIDIA & Wayland WebKitGTK fix: Prevents blank-screen rendering crashes
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // Force GTK to draw its own menu bar instead of delegating to the global
    // menu proxy (affects Cinnamon, MATE, and Ubuntu/Unity environments).
    std::env::set_var("UBUNTU_MENUPROXY", "");
    std::env::set_var("APPMENU_DISPLAY_BOTH", "1");

    wafy_lib::run(single_instance_listener);
}
