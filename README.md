# FERI Dashboard

Timetable (Wise Timetable iCal), deadlines and Obsidian-compatible notes for FERI students. Tauri 2 + Svelte 5.

- `linux/`: the app (builds for Linux and Windows; the folder name is historical). See [linux/README.md](linux/README.md).
- `android/`: planned.
- [PLAN.md](PLAN.md), [DESIGN.md](DESIGN.md)

## Windows

Built by GitHub Actions (`.github/workflows/windows.yml`). Download from the latest run's artifacts, or from Releases after pushing a `v*` tag:

- `FERI Dashboard_x.y.z_x64-setup.exe`: installer
- `feri-dashboard.exe`: portable

Obsidian is not required. On first run use Settings → Choose folder… and Create starter vault to get the empty folder layout (subjects, templates, index notes). Default notes folder: `%USERPROFILE%\Documents\FERI\Zapiski\Sola`. The `.exe` is unsigned, so SmartScreen warns on first run.
