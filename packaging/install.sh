#!/usr/bin/env bash
# Installs Sheetz for the current user (binary, icon and desktop entry).
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
prefix="${PREFIX:-$HOME/.local}"

echo "Building release binary…"
cargo build --release --manifest-path "$repo/Cargo.toml"

install -Dm755 "$repo/target/release/sheetz" "$prefix/bin/sheetz"
install -Dm644 "$repo/packaging/sheetz.svg" \
  "$prefix/share/icons/hicolor/scalable/apps/sheetz.svg"
install -Dm644 "$repo/packaging/sheetz.desktop" \
  "$prefix/share/applications/sheetz.desktop"
install -Dm644 "$repo/keymap.toml" "$HOME/.config/sheetz/keymap.toml"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$prefix/share/applications" || true
fi

echo "Installed to $prefix/bin/sheetz"
echo "Keymap installed at ~/.config/sheetz/keymap.toml — edit to taste."

# Nothing to do for assistants: Sheetz registers itself with any MCP clients it
# finds the first time it runs.
