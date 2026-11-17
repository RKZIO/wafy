# Wafy - Ultra-Lightweight Web Messaging Client for Linux

## 1. Project Overview & Motivation
**Wafy** is an open-source, ultra-lightweight desktop messaging client for Linux designed to wrap WhatsApp Web with minimal resource consumption and first-class desktop integration.

### The Problem with Existing Alternatives (e.g. ZapZap)
Existing community clients like **ZapZap** are built with Python, PyQt6, and QtWebEngine (Chromium-based), leading to:
- High baseline memory consumption (~500MB – 1GB+).
- Rapid memory inflation up to 1.5GB–2GB over long sessions.
- Process sprawl (multiple Chromium helper processes + Python runtime).

### The Wafy Solution
Wafy is built with **Tauri v2 + Rust + WebKitGTK**, designed to:
- Idle around **~150MB–250MB RAM** (a 70%+ reduction compared to ZapZap/Electron).
- Provide sub-second cold startup times.
- Integrate deeply with Linux desktop standards (system tray, unread badges, D-Bus notifications, MPRIS/media, Wayland/X11 support).
- Remain completely trademark-safe and open-source friendly.

---

## 2. Technology Stack & Comparison

| Component | ZapZap | Electron Wrappers | Wafy (Proposed) |
| :--- | :--- | :--- | :--- |
| **Backend / Host** | Python + PyQt6 | Node.js / Chromium | **Rust (Tauri v2)** |
| **Web Rendering Engine** | QtWebEngine (Chromium) | Chromium | **WebKitGTK (system shared library)** |
| **Process Model** | Multi-process Chromium + Python VM | Multi-process Chromium | **Single native binary + WebKit processes** |
| **Idle Memory (RAM)** | ~500MB – 1.2GB | ~600MB – 1.2GB | **~150MB – 250MB** |
| **Binary Footprint** | ~100MB+ (larger in Flatpak) | ~150MB+ | **~10MB – 15MB** |
| **Cold Startup Time** | 2 – 5 seconds | 2 – 4 seconds | **< 0.5 seconds** |

---

## 3. High-Level Architecture

```
┌────────────────────────────────────────────────────────┐
│                   Rust Host (Tauri v2)                 │
│  - System Tray Icon & Unread Counter Badge            │
│  - Minimize to Tray on Close ('X' button)             │
│  - Native Linux Desktop Notifications (D-Bus)         │
│  - NVIDIA / Wayland Compatibility Guardrails          │
│  - Session State & Cache Configuration                │
└──────────────────────────┬─────────────────────────────┘
                           │ IPC & Injected Scripts
┌──────────────────────────▼─────────────────────────────┐
│              WebKitGTK Embedded Webview                │
│  - Loads https://web.whatsapp.com                      │
│  - Spoofed Desktop Safari / Chrome User-Agent          │
│  - MutationObserver JS: watches unread message badges  │
│  - Custom CSS: theme synchronization and clean styling │
│  - GStreamer: audio playback and mic for voice notes   │
└────────────────────────────────────────────────────────┘
```

---

## 4. Key Features

1. **System Tray Integration:**
   - Minimizing or clicking the window close button (`X`) minimizes to the system tray.
   - Tray context menu: Show/Hide, Toggle Mute, Reload, Quit.
2. **Dynamic Unread Badge:**
   - Injected lightweight JavaScript observer tracks unread message count and updates the tray icon badge.
3. **Native Desktop Notifications:**
   - Translates WhatsApp HTML5 notifications into system notifications via `libnotify` / D-Bus.
4. **Hardware & Graphics Crash Prevention:**
   - Auto-applies `WEBKIT_DISABLE_DMABUF_RENDERER=1` when necessary to prevent blank-screen crashes on NVIDIA/Wayland drivers.
5. **Memory Optimizations:**
   - Uses WebKit's document viewer cache model (`WebKitCacheModelDocumentViewer`) to bound memory growth.
   - Throttles background animations and timers when the window is hidden/minimized.
   - Fast session persistence stored in `~/.config/wafy`.

---

## 5. Detailed Implementation Roadmap

### Phase 1: Environment Verification & Scaffolding
- Verify system development dependencies:
  - `rustc`, `cargo`
  - `libwebkit2gtk-4.1-dev` (or `libwebkit2gtk-4.0-dev`)
  - `libgtk-3-dev`
  - `libayatana-appindicator3-dev` (for system tray support)
  - `gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, `gstreamer1.0-pulseaudio` (for audio & voice messages)
- Initialize Tauri v2 project structure without heavy JS frontends (pure vanilla JS/HTML for local controls + remote webview).
- Configure permissions, window defaults, and data directory persistence (`~/.config/wafy`).

### Phase 2: Webview Configuration & Compatibility
- Point Tauri webview to `https://web.whatsapp.com`.
- Configure Desktop Safari / Chrome User-Agent string to satisfy WhatsApp Web compatibility checks.
- Add startup environment checks for NVIDIA/Wayland compatibility flags.

### Phase 3: Desktop Integration & Scripts
- Implement system tray with state toggles (show/hide).
- Inject `badge-observer.js` into WhatsApp Web:
  - Monitors the document title and unread badge DOM elements.
  - Emits Tauri IPC events (`tauri://event`) with current unread count.
  - Updates tray icon dynamically (e.g. badge counter overlay).
- Route HTML5 web notifications to native D-Bus desktop notifications.

### Phase 4: Memory Tuning & Media Support
- Configure WebKit cache limits and document viewing policies.
- Test voice notes, microphone permissions, and file drag-and-drop.
- Inject minimal custom CSS for theme alignment (Dark/Light).

### Phase 5: Build & Packaging
- Build optimized release binary with LTO (`lto = true`, `codegen-units = 1`, `strip = true`).
- Generate high-performance `.deb` Debian package for installation (`dpkg -i`).
- Create desktop entry (`wafy.desktop`) with high-resolution app icon and system launcher integration.
