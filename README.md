# Wafy 🌿

An ultra-lightweight, high-performance desktop messaging client for Linux built with **Tauri v2 + Rust + WebKitGTK**.

Zero Electron bloat, zero Python VM overhead, and zero Chromium multi-process sprawl.

---

## 🚀 Why Wafy?

Existing desktop clients for WhatsApp on Linux (such as ZapZap or Electron-based wrappers) often consume **600MB to 1.5GB+ of RAM** because they bundle or run heavy Chromium processes.

**Wafy** solves this by:
* Compiling to a native **Rust (Tauri v2)** binary.
* Utilizing the host operating system's native **WebKitGTK** engine.
* Idling at just **~150MB – 250MB RAM** (a **75%+ reduction** in resource usage).
* Providing instant, sub-second startup times (< 0.5s).
* Requiring **zero runtime dependencies** from end-users (no Python or Node.js needed).

---

## ✨ Features

* **Native System Tray Integration:** 
  * **Left Click:** Immediately opens, unminimizes, and focuses Wafy.
  * **Right Click:** Opens the context menu (Open, Reload, Quit).
  * Dynamic unread message count in the tray tooltip.
* **Close to Tray:** Closing the window (`X`) minimizes to the system tray so you never miss a message.
* **Native Desktop Notifications:** Direct integration with Linux desktop notifications via D-Bus (`libnotify`).
* **Hardware & Driver Guardrails:** Auto-detects and bypasses WebKit DMABUF renderer conflicts (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) to prevent Wayland and NVIDIA crashes.
* **Voice Notes & Microphone Support:** Native WebAudio / GStreamer integration with microphone permissions.
* **Custom Clean Styling:** Responsive slim scrollbars and automatic filtering of mobile download promotional banners.

---

## 📥 Download & Installation

### Option 1: Download from GitHub Releases (Recommended)

1. Head over to the **[Releases Page](https://github.com/RKZIO/wafy/releases)**.
2. Download the latest `.deb` package (e.g. `Wafy_*_amd64.deb`).
3. Open your terminal in the download directory and run:
   ```bash
   sudo dpkg -i Wafy_*_amd64.deb
   sudo apt-get install -f   # Installs system webkit2gtk libraries if missing
   ```
4. Launch **Wafy** from your application menu or by running `wafy` in your terminal.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + R` / `F5` | Reload WhatsApp Web |
| `Ctrl + Q` | Quit Wafy completely |

---

## 🛠️ Building from Source

### Prerequisites:
```bash
# On Debian / Ubuntu / Linux Mint:
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

### Build the `.deb` package:
```bash
git clone https://github.com/RKZIO/wafy.git
cd wafy
npm install
npx @tauri-apps/cli build --bundles deb
```

The resulting package will be generated at:
`src-tauri/target/release/bundle/deb/Wafy_*_amd64.deb`

---

## 📁 Project Structure

```
wafy/
├── .github/workflows/         # Automated GitHub Actions build & release pipeline
├── src/                       # Frontend webview injection layer
│   ├── index.html             # Initial splash & webview anchor
│   ├── badge_observer.js      # MutationObserver extracting unread counts & bridging IPC
│   └── style.css              # Custom slim scrollbars & responsive CSS
├── src-tauri/                 # Native Rust backend (Tauri v2)
│   ├── Cargo.toml             # Rust dependencies (tauri v2, serde, notifications)
│   ├── tauri.conf.json        # Tauri v2 window, tray, and Debian bundle settings
│   ├── build.rs               # Tauri build script
│   ├── icons/                 # High-resolution application & tray icons
│   └── src/
│       ├── main.rs            # Entry point with NVIDIA/Wayland driver fixes
│       └── lib.rs             # Tray builder, click events, close-to-tray, and IPC handlers
├── assets/                    # Scalable vector SVG icons and desktop specification
└── package.json               # Node tooling definition for Tauri CLI
```

---

## 📄 License
MIT License. Open-source and free for all Linux users.

*Disclaimer: Wafy is an unofficial open-source project and is not affiliated with or endorsed by WhatsApp or Meta.*
