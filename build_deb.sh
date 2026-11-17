#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_DIR}"

echo "=================================================="
echo " Building Wafy (Tauri v2 + Rust) Debian Package "
echo "=================================================="

# Check for Cargo / Rust
if command -v cargo >/dev/null 2>&1; then
    CARGO_BIN="cargo"
elif [ -f "${HOME}/.cargo/bin/cargo" ]; then
    CARGO_BIN="${HOME}/.cargo/bin/cargo"
    export PATH="${HOME}/.cargo/bin:${PATH}"
else
    echo "[-] Error: Cargo is not found in PATH or ~/.cargo/bin"
    echo "    Please install Rust via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "[+] Using Cargo: $("${CARGO_BIN}" --version)"

# Ensure Tauri CLI is available
if ! "${CARGO_BIN}" tauri --version >/dev/null 2>&1; then
    if command -v npx >/dev/null 2>&1; then
        echo "[+] Using npx @tauri-apps/cli to build Debian package..."
        npx @tauri-apps/cli build --bundles deb
    else
        echo "[+] Installing cargo-tauri CLI..."
        "${CARGO_BIN}" install tauri-cli --version "^2.0.0" --locked
        "${CARGO_BIN}" tauri build --bundles deb
    fi
else
    echo "[+] Running cargo tauri build --bundles deb..."
    "${CARGO_BIN}" tauri build --bundles deb
fi

DEB_OUTPUT=$(find "${PROJECT_DIR}/src-tauri/target/release/bundle/deb" -name "*.deb" 2>/dev/null | head -n 1 || true)

if [ -n "${DEB_OUTPUT}" ] && [ -f "${DEB_OUTPUT}" ]; then
    cp "${DEB_OUTPUT}" "${PROJECT_DIR}/wafy_1.0.0_amd64.deb"
    echo "=================================================="
    echo " [✓] Successfully built Debian package:"
    echo "     ${PROJECT_DIR}/wafy_1.0.0_amd64.deb"
    echo "=================================================="
    ls -lh "${PROJECT_DIR}/wafy_1.0.0_amd64.deb"
else
    echo "[-] Note: Bundle output not found in target directory."
fi
