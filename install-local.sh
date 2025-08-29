#!/usr/bin/env bash
set -euo pipefail
cargo build --release
install -Dm755 ./target/release/spliterma "$HOME/.local/bin/spliterma"
install -Dm644 data/com.spliterma.app.desktop "$HOME/.local/share/applications/com.spliterma.app.desktop"
install -Dm644 data/com.spliterma.app.metainfo.xml "$HOME/.local/share/metainfo/com.spliterma.app.metainfo.xml"
for dir in data/icons/hicolor/*/apps; do
  size="$(basename "$(dirname "$dir")")"
  install -Dm644 "$dir/com.spliterma.app.png" "$HOME/.local/share/icons/hicolor/$size/apps/com.spliterma.app.png"
done
gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" -f || true
gtk4-update-icon-cache "$HOME/.local/share/icons/hicolor" -f || true
update-desktop-database "$HOME/.local/share/applications" || true
echo "Installed. Run: spliterma  (or find it in the app grid)"
