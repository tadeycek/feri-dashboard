<script>
  import { onMount } from "svelte";
  import { app } from "./lib/store.svelte.js";
  import Rail from "./components/Rail.svelte";
  import Today from "./views/Today.svelte";
  import Week from "./views/Week.svelte";
  import Roki from "./views/Roki.svelte";
  import Workspace from "./components/vault/Workspace.svelte";
  import Settings from "./components/Settings.svelte";
  import RokDialog from "./components/RokDialog.svelte";
  import Toast from "./components/Toast.svelte";

  onMount(() => app.boot());

  const keys = { 1: "today", 2: "week", 3: "deadlines", 4: "notes" };

  function onKey(e) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const t = e.target;
    if (t instanceof HTMLElement && (t.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(t.tagName))) return;
    if (document.querySelector("dialog[open]")) return;
    if (keys[e.key]) app.go(keys[e.key]);
    else if (e.key === "r") app.refresh();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="shell" class:compact={app.view === "notes"}>
  <Rail />
  <main class="main" class:flush={app.view === "notes"}>
    {#if !app.ready}
      <p class="muted boot">Loading timetable …</p>
    {:else if app.view === "today"}
      <Today />
    {:else if app.view === "week"}
      <Week />
    {:else if app.view === "deadlines"}
      <Roki />
    {:else}
      <Workspace />
    {/if}
  </main>
</div>

<Settings />
<RokDialog />
<Toast />

<style>
  .shell { display: grid; grid-template-columns: var(--rail-w, 208px) minmax(0, 1fr); height: 100vh; }
  .shell.compact { --rail-w: 60px; }
  .main { overflow: auto; padding: 28px 36px 44px; min-width: 0; }
  .main.flush { padding: 0; overflow: hidden; }
  .boot { padding: 40px 0; }
</style>
