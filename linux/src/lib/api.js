import { invoke } from "@tauri-apps/api/core";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call(cmd, args) {
  if (inTauri) return invoke(cmd, args);
  // Plain browser (vite dev server): use demo data so the UI can be developed and screenshotted.
  if (import.meta.env.DEV) {
    const mock = await import("./mock.js");
    return mock.call(cmd, args);
  }
  throw new Error("The app only works inside the Tauri window.");
}

export const subjects = () => call("get_subjects");
export const getSettings = () => call("get_settings");
export const saveSettings = (settings) => call("save_settings", { settings });
export const loadCached = () => call("load_cached");
export const syncTimetable = () => call("sync_timetable");
export const listRoki = () => call("list_roki");
export const createRok = (rok) => call("create_rok", { rok });
export const toggleRok = (file) => call("toggle_rok", { file });
export const saveNote = (note) => call("save_note", { note });
export const recentNotes = () => call("recent_notes");
export const createStarterVault = (path) => call("create_starter_vault", { path });
/** Native folder picker. Resolves to the chosen path, or null if cancelled (or in the browser demo). */
export async function pickFolder(current) {
  if (!inTauri) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({ directory: true, multiple: false, defaultPath: current || undefined, title: "Choose your notes folder" });
}
export const openInObsidian = (path) => call("open_in_obsidian", { path });

// Background behaviour
export const getAutostart = () => call("get_autostart");
export const setAutostart = (enabled) => call("set_autostart", { enabled });
/** Subscribe to an event sent by the Rust side (tray actions, background refresh). No-op in the browser demo. */
export async function onEvent(name, callback) {
  if (!inTauri) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen(name, callback);
}

// Vault workspace (Obsidian-style)
export const vaultTree = () => call("vault_tree");
export const readNote = (path) => call("read_note", { path });
export const writeNote = (path, content) => call("write_note", { path, content });
export const createNote = (path, content = "") => call("create_note", { path, content });
export const createFolder = (path) => call("create_folder", { path });
/** Resolves to [newPath, number of other notes whose links were updated]. */
export const renamePath = (from, to) => call("rename_path", { from, to });
export const trashPath = (path) => call("trash_path", { path });
export const searchVault = (query) => call("search_vault", { query });
export const backlinks = (path) => call("backlinks", { path });
export const openExternal = (url) => call("open_external", { url });
