#!/usr/bin/env bash
set -euo pipefail
SRC="./spliterma.png"
OUT="./data/icons/hicolor"
SIZES=(16 32 48 64 128 256)
command -v magick >/dev/null || { echo "Install ImageMagick (magick)"; exit 1; }
[ -f "$SRC" ] || { echo "Missing $SRC"; exit 1; }
for s in "${SIZES[@]}"; do
  mkdir -p "$OUT/${s}x${s}/apps"
  magick "$SRC" -resize "${s}x${s}" -background none -gravity center -extent "${s}x${s}" \
    "$OUT/${s}x${s}/apps/com.spliterma.app.png"
done
echo "Icons written under $OUT"
