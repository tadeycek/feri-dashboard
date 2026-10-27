#!/usr/bin/env bash
# Adds FERI Dashboard to the application launcher (KDE/GNOME menu) for the current user.
#   ./install.sh              install or update the menu entry and icon
#   ./install.sh --uninstall  remove them again (the app itself is not touched)
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
desktop="$data/applications/feri-dashboard.desktop"
icons="$data/icons/hicolor"

refresh() {
  command -v update-desktop-database >/dev/null && update-desktop-database "$data/applications" 2>/dev/null || true
  command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$icons" 2>/dev/null || true
  # KDE rebuilds its menu cache by itself, this just makes it immediate.
  if command -v kbuildsycoca6 >/dev/null; then kbuildsycoca6 >/dev/null 2>&1 || true
  elif command -v kbuildsycoca5 >/dev/null; then kbuildsycoca5 >/dev/null 2>&1 || true; fi
}

if [ "${1:-}" = "--uninstall" ]; then
  rm -f "$desktop"
  for size in 32x32 128x128 256x256 512x512; do rm -f "$icons/$size/apps/feri-dashboard.png"; done
  refresh
  echo "Removed FERI Dashboard from the application launcher."
  exit 0
fi

bin="$here/src-tauri/target/release/feri-dashboard"
[ -x "$bin" ] || { echo "Build the app first: npx tauri build --no-bundle" >&2; exit 1; }

mkdir -p "$data/applications"
for pair in 32x32:32x32.png 128x128:128x128.png 256x256:128x128@2x.png 512x512:icon.png; do
  size="${pair%%:*}"; file="${pair#*:}"
  mkdir -p "$icons/$size/apps"
  cp "$here/src-tauri/icons/$file" "$icons/$size/apps/feri-dashboard.png"
done

cat > "$desktop" <<EOF
[Desktop Entry]
Type=Application
Name=FERI Dashboard
GenericName=Student dashboard
Comment=Timetable, deadlines and Obsidian notes for FERI
Exec="$bin"
Icon=feri-dashboard
Terminal=false
Categories=Education;
Keywords=FERI;timetable;schedule;urnik;notes;deadlines;
StartupNotify=true
StartupWMClass=feri-dashboard
EOF

refresh
echo "Installed. Look for \"FERI Dashboard\" in the application launcher (Education)."
