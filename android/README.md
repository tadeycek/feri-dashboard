# FERI Dashboard: Android

Not started. The plan is in [`../PLAN.md`](../PLAN.md), section 3.

Short version: Tauri 2 mobile, same Svelte frontend and Rust core as `../linux`, plus Kotlin for notifications, a home-screen widget and folder access to the Obsidian vault (synced with Syncthing).

Start it only after the Linux app is stable, because the Rust logic in `../linux/src-tauri/src/` (`ics.rs`, `store.rs`, `vault.rs`) is meant to be reused.
