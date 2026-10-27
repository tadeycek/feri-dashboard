<script>
  import { tick } from "svelte";
  import { vault, stemOf, dirOf } from "../../lib/vault.svelte.js";
  import { fold } from "../../lib/md.js";

  let q = $state("");
  let sel = $state(0);
  let input = $state();

  // Subsequence match, ranked: prefix > word start > substring > loose subsequence.
  function score(name, query) {
    const n = fold(name);
    const s = fold(query);
    if (!s) return 1;
    if (n.startsWith(s)) return 4;
    if (n.includes(` ${s}`)) return 3;
    if (n.includes(s)) return 2;
    let i = 0;
    for (const c of n) if (c === s[i]) i++;
    return i === s.length ? 1 : 0;
  }

  const list = $derived(
    vault.notes
      .map((e) => ({ e, s: score(stemOf(e.path), q) }))
      .filter((x) => x.s > 0)
      .sort((a, b) => b.s - a.s || b.e.modified - a.e.modified)
      .slice(0, 40)
      .map((x) => x.e),
  );

  $effect(() => {
    if (vault.switcher) {
      q = "";
      sel = 0;
      tick().then(() => input?.focus());
    }
  });
  $effect(() => { q; sel = 0; });

  const close = () => (vault.switcher = false);
  function go(e, newTab = false) {
    if (!e) return;
    close();
    vault.open(e.path, { newTab });
  }
  function key(e) {
    if (e.key === "ArrowDown") { e.preventDefault(); sel = Math.min(sel + 1, list.length - 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); sel = Math.max(sel - 1, 0); }
    else if (e.key === "Enter") { e.preventDefault(); go(list[sel], e.ctrlKey || e.metaKey); }
    else if (e.key === "Escape") close();
  }
</script>

{#if vault.switcher}
  <div class="scrim" role="presentation" onclick={close}>
    <div class="box" role="dialog" aria-label="Quick switcher" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={key}>
      <input bind:this={input} bind:value={q} placeholder="Find a note…" spellcheck="false" aria-label="Find a note" />
      <ul role="listbox">
        {#each list as e, i (e.path)}
          <li role="option" aria-selected={i === sel}>
            <button class:on={i === sel} onmouseenter={() => (sel = i)} onclick={(ev) => go(e, ev.ctrlKey || ev.metaKey)}>
              <span>{stemOf(e.path)}</span><em>{dirOf(e.path)}</em>
            </button>
          </li>
        {:else}
          <li class="none muted">No results.</li>
        {/each}
      </ul>
      <p class="hint muted">↑↓ select · Enter open · Ctrl+Enter new tab · Esc close</p>
    </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; z-index: 70; background: rgb(0 0 0 / 0.45); display: grid; justify-items: center; align-items: start; padding-top: 12vh; }
  .box { width: min(620px, calc(100vw - 40px)); background: var(--panel); border: 1px solid var(--line); border-radius: 12px; box-shadow: var(--shadow); padding: 10px; display: grid; gap: 8px; animation: pop 0.12s ease-out; }
  @keyframes pop { from { opacity: 0; } }
  input { font-size: 17px; padding: 10px 12px; }
  ul { list-style: none; margin: 0; padding: 0; max-height: 50vh; overflow: auto; }
  li button { width: 100%; display: flex; justify-content: space-between; gap: 12px; text-align: left; padding: 7px 12px; border: 0; background: transparent; border-radius: 6px; font-size: 15px; }
  li button.on { background: var(--accent); color: var(--accent-ink); }
  li button em { font-style: normal; font-size: 13px; opacity: 0.7; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .none { padding: 10px 12px; }
  .hint { font-size: 13px; padding: 0 4px; }
</style>
