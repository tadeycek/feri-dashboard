<script>
  import { app } from "../lib/store.svelte.js";
  import { ago, dayKey, dayMonth } from "../lib/time.js";
  import { colorOf } from "../lib/subjects.js";

  const guess = app.guessSubject();
  let predmet = $state(guess.predmet);
  let kind = $state(guess.kind);
  let title = $state("");
  let body = $state("");
  let saving = $state(false);
  let bodyEl = $state();

  const suggested = $derived(`${kind === "Vaje" ? "Vaje" : "Predavanje"}, ${dayMonth(app.now)}`);
  const key = $derived(app.keyOf(predmet));

  // A class picked on Danes or Teden prefills subject and kind once.
  $effect(() => {
    if (app.draft) {
      predmet = app.draft.predmet || predmet;
      kind = app.draft.kind;
      app.draft = null;
      queueMicrotask(() => bodyEl?.focus());
    }
  });

  async function save() {
    if (saving || !body.trim() || !predmet) return;
    saving = true;
    const info = await app.saveNote({ predmet, kind, title: (title.trim() || suggested), body, datum: dayKey(app.now) });
    saving = false;
    if (info) {
      title = "";
      body = "";
      bodyEl?.focus();
    }
  }

  function onKey(e) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      save();
    }
  }
</script>

<div class="page">
  <header class="head">
    <h1 class="num">Zapiski</h1>
    <p class="muted">Napiši zapisek tukaj, shrani se kot navadna beležka v Obsidianovem vaultu.</p>
  </header>

  {#if app.vaultError}<p class="banner">{app.vaultError}</p>{/if}

  <div class="grid">
    <section class="panel editor" style:--c={colorOf(key)}>
      <div class="chips" role="radiogroup" aria-label="Predmet">
        {#each app.subjects as s (s.key)}
          <button class="chip" role="radio" aria-checked={predmet === s.name} class:on={predmet === s.name} style:--c={colorOf(s.key)} onclick={() => (predmet = s.name)}>
            <span class="dot"></span>{s.name}
          </button>
        {/each}
      </div>

      <div class="seg" role="radiogroup" aria-label="Vrsta zapiska">
        {#each ["Predavanja", "Vaje"] as k (k)}
          <button role="radio" aria-checked={kind === k} class:on={kind === k} onclick={() => (kind = k)}>{k}</button>
        {/each}
      </div>

      <label class="field">
        <span>Naslov</span>
        <input type="text" bind:value={title} placeholder={suggested} onkeydown={onKey} />
      </label>

      <label class="field grow">
        <span>Zapisek</span>
        <textarea bind:this={bodyEl} bind:value={body} rows="9" placeholder="Kaj si se naučil, kaj je ostalo nejasno …" onkeydown={onKey}></textarea>
      </label>

      <div class="foot">
        <p class="muted small">Mapa: Predmeti/{predmet || "…"}/{kind}. Bližnjica: Ctrl + Enter.</p>
        <button class="btn" onclick={save} disabled={saving || !body.trim() || !predmet}>Shrani v Obsidian</button>
      </div>
    </section>

    <aside class="panel recent">
      <h2 class="section-title">Nedavni zapiski</h2>
      {#if app.notes.length}
        <ul>
          {#each app.notes as n (n.path)}
            <li style:--c={colorOf(app.keyOf(n.predmet))}>
              <span class="dot"></span>
              <div>
                <p class="name">{n.title}</p>
                <p class="sub">{n.predmet} · {n.kind} · {ago(new Date(n.modified * 1000), app.now)}</p>
              </div>
              <button class="btn ghost small" onclick={() => app.openNote(n.path)}>Odpri</button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted">Še ni zapiskov. Prvi zapisek se bo pojavil tukaj.</p>
      {/if}
    </aside>
  </div>
</div>

<style>
  .page { display: grid; gap: 22px; max-width: 1240px; }
  .head h1 { font-size: 54px; line-height: 0.95; font-weight: 700; }
  .head p { margin-top: 6px; }
  .grid { display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(280px, 1fr); gap: 22px; align-items: start; }

  .editor { padding: 22px 24px 20px; display: grid; gap: 16px; border-top: 6px solid var(--c); }
  .chips { display: flex; flex-wrap: wrap; gap: 8px; }
  .chip { display: inline-flex; align-items: center; gap: 8px; padding: 6px 13px; border-radius: 999px; border: 1px solid var(--line); background: var(--panel); font-weight: 600; font-size: 14px; color: var(--ink-2); transition: background 0.12s, border-color 0.12s; }
  .chip .dot { background: var(--c); }
  .chip:hover { border-color: var(--ink-3); }
  .chip.on { background: color-mix(in oklab, var(--c) 16%, var(--panel)); border-color: var(--c); color: var(--ink); }

  .seg { display: inline-flex; justify-self: start; padding: 3px; border-radius: var(--r-sm); background: var(--panel-2); border: 1px solid var(--line-2); }
  .seg button { border: 0; background: transparent; padding: 6px 16px; border-radius: 5px; font-weight: 600; color: var(--ink-3); }
  .seg button.on { background: var(--panel); color: var(--ink); box-shadow: 0 1px 3px rgb(10 16 30 / 0.15); }

  .field.grow textarea { min-height: 200px; font-size: 15.5px; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
  .foot .btn { white-space: nowrap; flex: none; }
  .small { font-size: 13px; }

  .recent { padding: 20px 22px 16px; display: grid; gap: 12px; }
  .recent ul { list-style: none; margin: 0; padding: 0; display: grid; }
  .recent li { display: grid; grid-template-columns: 9px minmax(0, 1fr) auto; gap: 12px; align-items: center; padding: 10px 0; border-top: 1px solid var(--line-2); }
  .recent li:first-child { border-top: 0; }
  .recent .dot { background: var(--c); }
  .name { font-weight: 600; line-height: 1.3; }
  .sub { font-size: 13px; color: var(--ink-3); }

  @media (max-width: 1080px) { .grid { grid-template-columns: minmax(0, 1fr); } }
</style>
