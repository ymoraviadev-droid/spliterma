# Spliterma

A simple **GTK4 + VTE** tiling terminal for Linux.  
Split panes horizontally/vertically, color-code each pane, rename titles, and **Save/Load layouts** as JSON. Each terminal restores its **working directory** and color.

> Built with Rust, `gtk4`, and `vte4`.

---

## Features

- Split **Horizontal** / **Vertical**
- Per-pane **title** (double-click to rename)
- Per-pane **color** (picker)
- **Save layout** to JSON
- **Load layout** from JSON
- Remembers **working directory** per terminal (VTE OSC 7)
- Context menu (right-click)
- Menu shortcuts:
  - **Ctrl+S** – Save Layout
  - **Ctrl+O** – Load Layout

---

## Requirements

### Rust
```bash
# If you don't have Rust yet
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

### System packages

**Fedora:**
```bash
sudo dnf install -y gtk4-devel vte291-devel glib2-devel pkgconf-pkg-config ImageMagick vte-profile
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y libgtk-4-dev libvte-2.91-dev libglib2.0-dev pkg-config build-essential imagemagick
```

> `vte-profile` (Fedora) or your distro’s VTE bash integration helps persist the **live working directory**.

---

## Build & Run

```bash
# Clone
git clone https://github.com/yourname/spliterma.git
cd spliterma

# Debug
cargo run

# Release
cargo build --release
./target/release/spliterma
```

---

## Usage

- **Right-click** a terminal for:
  - Split Horizontal / Vertical
  - Save Layout
  - Load Layout
  - Stop Terminal
- **Double-click** the title to rename.
- **Click** the color dot to change pane color.
- **Ctrl+S** / **Ctrl+O** via the app menu.

---

## Layout JSON (example)

```json
{
  "version": "1.0",
  "root": {
    "name": "Split",
    "color_index": 0,
    "working_dir": "",
    "split_type": "Horizontal",
    "children": [
      {
        "name": "Terminal 1",
        "color_index": 0,
        "working_dir": "/home/you",
        "split_type": null,
        "children": []
      },
      {
        "name": "Backend",
        "color_index": 6,
        "working_dir": "/home/you/projects/api",
        "split_type": null,
        "children": []
      }
    ]
  }
}
```

---

## App Icon (transparent PNG)

1) Put a **transparent** PNG named `spliterma.png` in the **project root**.  
2) Generate & install icon sizes for app ID **`com.spliterma.app`**:

```bash
cat > make_app_icon.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

APP_ID="com.spliterma.app"
SRC="./spliterma.png"             # transparent PNG in project root
OUT="/tmp/spliterma-icons"
SIZES=(16 32 48 64 128 256)

command -v magick >/dev/null || { echo "ImageMagick (magick) not found"; exit 1; }
[ -f "$SRC" ] || { echo "Source PNG not found at: $SRC"; exit 1; }

rm -rf "$OUT"; mkdir -p "$OUT"
for s in "${SIZES[@]}"; do
  magick "$SRC" -resize "${s}x${s}" -background none -gravity center -extent "${s}x${s}" "$OUT/${s}.png"
done
cp "$OUT/256.png" "$OUT/${APP_ID}.png"

for s in "${SIZES[@]}"; do
  mkdir -p "$HOME/.local/share/icons/hicolor/${s}x${s}/apps"
  cp "$OUT/${s}.png" "$HOME/.local/share/icons/hicolor/${s}x${s}/apps/${APP_ID}.png"
done

gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" -f || true
gtk4-update-icon-cache "$HOME/.local/share/icons/hicolor" -f || true

echo "Installed icons for ${APP_ID}:"
ls -l "$HOME/.local/share/icons/hicolor/"*"/apps/${APP_ID}.png" 2>/dev/null || true
EOF

chmod +x make_app_icon.sh
bash make_app_icon.sh
```

3) (Optional) Desktop launcher:

Create `~/.local/share/applications/com.spliterma.app.desktop`:

```
[Desktop Entry]
Name=Spliterma
Exec=/full/path/to/your/binary
Icon=com.spliterma.app
Type=Application
Terminal=false
Categories=Utility;Development;
StartupWMClass=com.spliterma.app
```

Update the desktop DB (optional):
```bash
update-desktop-database ~/.local/share/applications || true
```

> Ensure your code uses:
> ```rust
> let app = gtk::Application::builder()
>     .application_id("com.spliterma.app")
>     .build();
> ```

---

## 🗂️ Project layout (simplified)

```
src/
  main.rs
  app.rs
  constants.rs
  ui/
    mod.rs
    terminal.rs      # terminal widget + title + color picker + context menu
    split.rs         # split/stop/replace logic
  layout/
    mod.rs
    types.rs         # TerminalLayout / SplitType / SavedLayout
    extract.rs       # extract GTK tree -> TerminalLayout (save)
    persist.rs       # save/load JSON, build layout (load)
  util/
    mod.rs
    errors.rs        # error dialog helper
    ids.rs           # AtomicUsize terminal counter
```

---

## Notes

- **Working directory** persistence uses VTE’s OSC 7. On Fedora install `vte-profile`. On other distros, ensure your shell sources the VTE integration that ships with VTE.
- If the icon doesn’t show immediately in launchers, log out/in or restart the shell once.

---

## License

MIT

---

## Contribute

PRs welcome:
- Bug fixes (layout edge cases)
- Improved icon / theming
- Config & settings
- Packaging (Flatpak, RPM/DEB)
