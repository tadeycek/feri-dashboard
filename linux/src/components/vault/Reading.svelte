<script>
  import { splitFrontmatter, renderMarkdown, toggleTask } from "../../lib/md.js";
  import { vault } from "../../lib/vault.svelte.js";
  import { app } from "../../lib/store.svelte.js";
  import * as api from "../../lib/api.js";

  let { path, content } = $props();

  const parts = $derived(splitFrontmatter(content));
  const html = $derived(renderMarkdown(parts.body, { resolve: (n) => !!vault.resolve(n) }));

  const isLink = (v) => /^\[\[.+\]\]$/.test(v);
  const linkName = (v) => v.slice(2, -2).split("|")[0];
  const items = (v) => (Array.isArray(v) ? v : v ? [v] : []);

  async function onclick(e) {
    const el = e.target;
    if (!(el instanceof Element)) return;

    const box = el.closest("input.task");
    if (box) {
      const all = [...e.currentTarget.querySelectorAll("input.task")];
      vault.setContent(path, toggleTask(content, all.indexOf(box)));
      return;
    }
    const wiki = el.closest("a.wikilink");
    if (wiki) {
      e.preventDefault();
      vault.openByName(wiki.dataset.link, path);
      return;
    }
    const base = el.closest(".embed-card[data-base]");
    if (base) {
      const hit = vault.entries.find((x) => x.path.endsWith(`/${base.dataset.base}`) || x.path === base.dataset.base);
      if (hit) app.openNote(hit.path);
      return;
    }
    const a = el.closest("a[href]");
    if (a) {
      e.preventDefault();
      const href = a.getAttribute("href");
      if (href?.startsWith("#")) document.getElementById(decodeURIComponent(href.slice(1)))?.scrollIntoView({ behavior: "smooth" });
      else if (href) api.openExternal(href).catch((err) => app.notify(String(err?.message ?? err)));
    }
  }
</script>

<article class="reading" {onclick} onkeydown={(e) => e.key === "Enter" && onclick(e)} role="presentation">
  {#if parts.props.length}
    <section class="props" aria-label="Properties">
      <h2>Properties</h2>
      {#each parts.props as [key, value] (key)}
        <div class="prop">
          <span class="k">{key}</span>
          <span class="v">
            {#each items(value) as v, i (i)}
              {#if isLink(v)}
                <a class="wikilink pill" href="#" data-link={linkName(v)}>{linkName(v)}</a>
              {:else if Array.isArray(value)}
                <span class="pill">{v}</span>
              {:else}
                {v}
              {/if}
            {/each}
          </span>
        </div>
      {/each}
    </section>
  {/if}
  <div class="md">{@html html}</div>
</article>

<style>
  .reading { width: min(760px, 100%); margin: 0 auto; padding: 4px 24px 45vh; }

  .props { margin: 0 0 22px; padding: 4px 0 10px; border-bottom: 1px solid var(--line-2); }
  .props h2 { font: 600 13px var(--font-ui); color: var(--ink-3); margin-bottom: 6px; }
  .prop { display: grid; grid-template-columns: 140px minmax(0, 1fr); gap: 12px; padding: 3px 0; font-size: 14px; align-items: baseline; }
  .k { color: var(--ink-3); }
  .v { display: flex; flex-wrap: wrap; gap: 6px; }
  .pill { padding: 0 9px; border-radius: 999px; background: var(--panel-3); font-size: 13px; line-height: 1.6; }

  /* Rendered markdown */
  .md { font-size: 16px; line-height: 1.7; overflow-wrap: anywhere; }
  .md :global(h1) { font-size: 1.95em; line-height: 1.3; margin: 1.1em 0 0.4em; }
  .md :global(h2) { font-size: 1.6em; line-height: 1.3; margin: 1.1em 0 0.4em; }
  .md :global(h3) { font-size: 1.35em; margin: 1em 0 0.35em; }
  .md :global(h4) { font-size: 1.15em; margin: 1em 0 0.3em; }
  .md :global(h5), .md :global(h6) { font-size: 1em; color: var(--ink-2); margin: 1em 0 0.3em; }
  .md :global(h1:first-child), .md :global(h2:first-child) { margin-top: 0; }
  .md :global(p) { margin: 0 0 0.9em; }
  .md :global(a) { color: var(--accent); text-decoration: none; }
  .md :global(a:hover) { text-decoration: underline; }
  .md :global(a.unresolved) { opacity: 0.6; }
  .md :global(ul), .md :global(ol) { padding-left: 1.5em; margin: 0 0 0.9em; }
  .md :global(li) { margin: 0.15em 0; }
  .md :global(li:has(> input.task)) { list-style: none; margin-left: -1.3em; }
  .md :global(input.task) { accent-color: var(--accent); width: 15px; height: 15px; margin: 0 6px 0 0; vertical-align: -2px; cursor: pointer; }
  .md :global(code) { font-family: ui-monospace, "JetBrains Mono", monospace; font-size: 0.88em; background: var(--panel-3); padding: 1px 5px; border-radius: 4px; }
  .md :global(pre) { background: var(--panel-2); border: 1px solid var(--line-2); border-radius: 8px; padding: 12px 14px; overflow: auto; margin: 0 0 1em; }
  .md :global(pre code) { background: none; padding: 0; }
  .md :global(blockquote) { margin: 0 0 1em; padding: 2px 0 2px 16px; border-left: 3px solid var(--line); color: var(--ink-2); }
  .md :global(hr) { border: 0; border-top: 1px solid var(--line); margin: 1.6em 0; }
  .md :global(table) { border-collapse: collapse; margin: 0 0 1em; font-size: 15px; }
  .md :global(th), .md :global(td) { border: 1px solid var(--line); padding: 5px 12px; text-align: left; }
  .md :global(th) { background: var(--panel-2); }
  .md :global(img) { max-width: 100%; }

  .md :global(.callout) { --cc: var(--c-UVR); margin: 0 0 1em; border-radius: var(--r-lg); padding: 10px 14px 6px; background: color-mix(in oklab, var(--cc) 12%, var(--paper)); }
  .md :global(.callout.ok) { --cc: var(--c-PRO1); }
  .md :global(.callout.warn) { --cc: var(--c-SPO); }
  .md :global(.callout.bad) { --cc: var(--c-MAT); }
  .md :global(.callout.quote) { --cc: var(--ink-3); }
  .md :global(.callout-title) { font-weight: 700; color: var(--cc); margin-bottom: 4px; }
  .md :global(.callout-body > :last-child) { margin-bottom: 0.4em; }

  .md :global(.embed-card) { display: grid; gap: 2px; margin: 0 0 1em; padding: 10px 14px; border: 1px dashed var(--line); border-radius: 8px; background: var(--panel-2); font-size: 14px; cursor: default; }
  .md :global(.embed-card[data-base]) { cursor: pointer; }
  .md :global(.embed-card[data-base]:hover) { border-color: var(--accent); }
  .md :global(.embed-card b) { color: var(--ink-2); }
  .md :global(.embed-card em) { color: var(--ink-3); font-style: normal; }
</style>
