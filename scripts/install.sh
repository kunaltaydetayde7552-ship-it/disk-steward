#!/usr/bin/env bash
# Build and install Disk Steward for the current user (macOS only) via `cargo tauri build`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
CARGO="$(command -v cargo || echo "$HOME/.cargo/bin/cargo")"

if [ "$(uname -s)" != "Darwin" ]; then
    echo "Unsupported OS: $(uname -s). Disk Steward is macOS-only (APFS)." >&2
    exit 1
fi
if ! "$CARGO" tauri --version >/dev/null 2>&1; then
    echo "tauri-cli is required. Install it with: cargo install tauri-cli --version \"^2\" --locked" >&2
    exit 1
fi
command -v bun >/dev/null 2>&1 || { echo "bun is required to build the UI (https://bun.sh)" >&2; exit 1; }
if [ ! -x "$HOME/.cargo/bin/auto-compress-apfs" ]; then
    echo "Warning: ~/.cargo/bin/auto-compress-apfs not found; the app needs this engine to measure and compress." >&2
fi
export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
(cd ui && bun install --frozen-lockfile)

BUNDLE_DIR="target/release/bundle"

echo "Building the macOS app bundle (cargo tauri build --bundles app)..."
"$CARGO" tauri build --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
# the newest bundle wins: older ones can stay in target/
APP="$(ls -td "$BUNDLE_DIR"/macos/*.app 2>/dev/null | head -n 1)"
[ -n "$APP" ] || { echo "No .app produced under $BUNDLE_DIR/macos" >&2; exit 1; }
DEST="/Applications"
[ -w "$DEST" ] || DEST="$HOME/Applications"
mkdir -p "$DEST"
rm -rf "${DEST:?}/$(basename "$APP")"
cp -R "$APP" "$DEST/"
# ad-hoc signature keeps a stable identity for the macOS permission prompts (best effort)
if ! codesign --force --deep --sign - "$DEST/$(basename "$APP")"; then
    echo "Warning: ad-hoc codesign failed; macOS may re-ask permissions after each update" >&2
fi
echo "Installed to $DEST/$(basename "$APP")"
echo "Schedule the daily compression with: ~/.cargo/bin/auto-compress-apfs install-launchd"
