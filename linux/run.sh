#!/usr/bin/env bash
# Starts the FERI Dashboard. The timetable is fetched on every start.
# If the window opens blank (WebKitGTK + Wayland/NVIDIA), run: WEBKIT_DISABLE_DMABUF_RENDERER=1 ./run.sh
cd "$(dirname "$0")"
BIN=src-tauri/target/release/feri-dashboard
if [ ! -x "$BIN" ]; then
  echo "Ni zgrajeno. Zaženi: npm install && npx tauri build --no-bundle" >&2
  exit 1
fi
exec "$BIN" "$@"
