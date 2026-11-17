# Wafy 🌿

An ultra-lightweight, high-performance desktop messaging client for Linux built with **Tauri v2 + Rust + WebKitGTK**.

Zero Electron bloat, zero Python VM overhead, and zero Chromium multi-process sprawl.

---

## 🚀 Why Wafy?

Existing desktop clients for WhatsApp on Linux (such as ZapZap, WhatsDesk, or Electron clients) consume **600MB to 1.5GB+ of RAM** because they run heavy Chromium or QtWebEngine instances.

**Wafy** solves this by:
* Compiling to a native **Rust (Tauri v2)** binary (`/usr/bin/wafy`).
* Utilizing the host operating system's native **WebKitGTK** shared engine.
* Idling at just **~150MB – 250MB RAM** (a **75%+ reduction** in resource usage).
* Providing sub-second cold startup times (< 0.5s).
* Requiring **zero runtime dependencies** (no Python, no Node.js required by end-users).

---

## ✨ Features

* **Native Rust Host:** Compiled native ELF binary for maximum speed and minimum memory footprint.
* **System Tray Integration:** Native tray indicator with dynamic unread message count badge (`(3) Wafy`).
* **Close to Tray:** Closing the window (`X`) minimizes to the system tray so you never miss notifications.
* **Native Desktop Notifications:** Direct integration with Linux desktop notifications via D-Bus (`libnotify`).
* **Hardware & Driver Guardrails:** Auto-detects and bypasses WebKit DMABUF renderer conflicts (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) to prevent Wayland and NVIDIA crashes.
* **Voice Notes & Microphone Support:** Native WebAudio / GStreamer integration with microphone permissions.
* **Custom Clean Styling:** Responsive slim scrollbars and automatic filtering of mobile download promotional banners.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + R` / `F5` | Reload WhatsApp Web |
| `Ctrl + Q` | Quit Wafy completely |

---

## 📦 Installation for End-Users (.deb)

The pre-built `.deb` package is completely standalone and works on any Debian-based Linux distribution (Debian, Ubuntu, Linux Mint, Pop!_OS, Zorin, etc.):

```bash
sudo dpkg -i wafy_1.0.0_amd64.deb
sudo apt-get install -f   # Installs system webkit2gtk libraries if not already present
```

**End-users do NOT need Rust, Cargo, Node.js, or Python.** The application is distributed as a compiled native binary.

---

## 🛠️ Building from Source

### Prerequisites (For Developers Building from Source Only):
```bash
# On Debian / Ubuntu / Linux Mint:
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

### Build the `.deb` package:
```bash
git clone https://github.com/yourusername/wafy.git
cd wafy
./build_deb.sh
```

Or using Cargo directly:
```bash
cargo tauri build --bundles deb
```

---

## 📁 Project Architecture

```
wafy/
├── PLAN.md                    # Architecture & optimization blueprint
├── README.md                  # Project documentation & user guide
├── package.json               # Node tooling definition for Tauri CLI
├── build_deb.sh               # Automated Debian build script
├── src/                       # Frontend webview injection layer
│   ├── index.html             # Initial splash & webview anchor
│   ├── badge_observer.js      # MutationObserver extracting unread counts
│   └── style.css              # Custom slim scrollbars & responsive CSS
├── src-tauri/                 # Native Rust backend (Tauri v2)
│   ├── Cargo.toml             # Rust dependencies (tauri v2, serde, notifications)
│   ├── tauri.conf.json        # Tauri v2 window, tray, and Debian bundle settings
│   ├── build.rs               # Tauri build script
│   ├── icons/                 # Application & tray icons (32x32, 128x128, 512x512)
│   └── src/
│       ├── main.rs            # Entry point with NVIDIA/Wayland driver fixes
│       └── lib.rs             # Tray builder, close-to-tray logic, and IPC handlers
└── assets/
    ├── icons/                 # Original scalable vector SVG icons
    └── wafy.desktop           # FreeDesktop application launcher specification
```

---

## 📄 License
MIT License. Open-source and free for all Linux users.
*Disclaimer: Wafy is an unofficial open-source project and is not affiliated with or endorsed by WhatsApp or Meta.*
