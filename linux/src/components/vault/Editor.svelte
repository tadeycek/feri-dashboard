<script>
  import { onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, drawSelection, highlightActiveLine, placeholder, Decoration, ViewPlugin, MatchDecorator } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { markdown, markdownLanguage, markdownKeymap } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, HighlightStyle, indentOnInput } from "@codemirror/language";
  import { autocompletion, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
  import { tags as t } from "@lezer/highlight";
  import { fold } from "../../lib/md.js";

  let { content, names = () => [], onchange, onlink, autofocus = false } = $props();

  let host;
  let view;

  const highlight = HighlightStyle.define([
    { tag: t.heading1, fontSize: "1.95em", fontWeight: "700", lineHeight: "1.3" },
    { tag: t.heading2, fontSize: "1.6em", fontWeight: "700", lineHeight: "1.3" },
    { tag: t.heading3, fontSize: "1.35em", fontWeight: "650", lineHeight: "1.35" },
    { tag: t.heading4, fontSize: "1.15em", fontWeight: "650" },
    { tag: [t.heading5, t.heading6], fontWeight: "650", color: "var(--ink-2)" },
    { tag: t.strong, fontWeight: "700" },
    { tag: t.emphasis, fontStyle: "italic" },
    { tag: t.strikethrough, textDecoration: "line-through", color: "var(--ink-3)" },
    { tag: t.link, color: "var(--accent)" },
    { tag: t.url, color: "var(--accent)", opacity: "0.8" },
    { tag: t.monospace, fontFamily: "ui-monospace, 'JetBrains Mono', monospace", fontSize: "0.9em", color: "var(--c-SPO)" },
    { tag: t.quote, color: "var(--ink-2)" },
    { tag: [t.processingInstruction, t.contentSeparator], color: "var(--ink-3)" },
    { tag: t.meta, color: "var(--ink-3)" },
  ]);

  const theme = EditorView.theme({
    "&": { height: "100%", color: "var(--ink)", backgroundColor: "transparent", fontSize: "16px" },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": { fontFamily: "var(--font-ui)", lineHeight: "1.7", display: "flex", justifyContent: "center", overflow: "auto" },
    ".cm-content": { flex: "0 1 760px", padding: "4px 24px 45vh", caretColor: "var(--ink)" },
    ".cm-line": { padding: "0" },
    ".cm-activeLine": { backgroundColor: "transparent" },
    ".cm-cursor": { borderLeftColor: "var(--ink)", borderLeftWidth: "2px" },
    ".cm-selectionBackground, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": { backgroundColor: "color-mix(in oklab, var(--accent) 35%, transparent)" },
    ".cm-placeholder": { color: "var(--ink-3)" },
    ".cm-wikilink": { color: "var(--accent)", cursor: "pointer", textDecoration: "none" },
    ".cm-wikilink:hover": { textDecoration: "underline" },
    ".cm-tooltip": { border: "1px solid var(--line)", backgroundColor: "var(--panel)", borderRadius: "8px", boxShadow: "var(--shadow)", overflow: "hidden" },
    ".cm-tooltip.cm-tooltip-autocomplete > ul": { fontFamily: "var(--font-ui)", maxHeight: "260px" },
    ".cm-tooltip-autocomplete > ul > li": { padding: "4px 12px" },
    ".cm-tooltip-autocomplete > ul > li[aria-selected]": { backgroundColor: "var(--accent)", color: "var(--accent-ink)" },
  });

  // [[wiki links]] get their own colour; Ctrl/Cmd+click opens them.
  const wikiMatcher = new MatchDecorator({ regexp: /\[\[[^\]\n]+\]\]/g, decoration: Decoration.mark({ class: "cm-wikilink" }) });
  const wikiPlugin = ViewPlugin.fromClass(
    class {
      constructor(v) { this.decorations = wikiMatcher.createDeco(v); }
      update(u) { this.decorations = wikiMatcher.updateDeco(u, this.decorations); }
    },
    { decorations: (p) => p.decorations },
  );

  function wikiSource(ctx) {
    const m = ctx.matchBefore(/\[\[[^\]\n|#]*/);
    if (!m) return null;
    const typed = fold(m.text.slice(2));
    const list = names()
      .map((n) => ({ n, f: fold(n) }))
      .filter((x) => x.f.includes(typed))
      .sort((a, b) => Number(b.f.startsWith(typed)) - Number(a.f.startsWith(typed)) || a.n.length - b.n.length)
      .slice(0, 30)
      .map((x) => x.n);
    return {
      from: m.from + 2,
      filter: false,
      options: list.map((n) => ({
        label: n,
        apply(v, _c, from, to) {
          const after = v.state.sliceDoc(to, to + 2);
          const insert = after === "]]" ? n : `${n}]]`;
          v.dispatch({ changes: { from, to, insert }, selection: { anchor: from + n.length + 2 } });
        },
      })),
    };
  }

  const wrap = (mark) => (v) => {
    const { from, to } = v.state.selection.main;
    const sel = v.state.sliceDoc(from, to);
    if (sel.startsWith(mark) && sel.endsWith(mark) && sel.length >= mark.length * 2) {
      v.dispatch({ changes: { from, to, insert: sel.slice(mark.length, -mark.length) } });
    } else {
      v.dispatch({ changes: { from, to, insert: mark + sel + mark }, selection: { anchor: from + mark.length, head: to + mark.length } });
    }
    return true;
  };

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: content,
        extensions: [
          history(),
          drawSelection(),
          highlightActiveLine(),
          indentOnInput(),
          closeBrackets(),
          markdown({ base: markdownLanguage }),
          syntaxHighlighting(highlight),
          wikiPlugin,
          autocompletion({ override: [wikiSource], icons: false }),
          placeholder("Start writing…"),
          EditorView.lineWrapping,
          theme,
          keymap.of([{ key: "Mod-b", run: wrap("**") }, { key: "Mod-i", run: wrap("*") }, ...markdownKeymap, ...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap, indentWithTab]),
          EditorView.domEventHandlers({
            mousedown(e, v) {
              if (!(e.ctrlKey || e.metaKey)) return false;
              const pos = v.posAtCoords({ x: e.clientX, y: e.clientY });
              if (pos == null) return false;
              const line = v.state.doc.lineAt(pos);
              for (const m of line.text.matchAll(/\[\[([^\]\n]+)\]\]/g)) {
                const start = line.from + m.index;
                if (pos >= start && pos <= start + m[0].length) {
                  e.preventDefault();
                  onlink?.(m[1].split("|")[0].split("#")[0].trim());
                  return true;
                }
              }
              return false;
            },
          }),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onchange?.(u.state.doc.toString());
          }),
        ],
      }),
    });
    if (autofocus) {
      view.focus();
      view.dispatch({ selection: { anchor: view.state.doc.length } });
    }
    return () => view.destroy();
  });

  // Pick up changes made elsewhere (e.g. the note was edited in Obsidian).
  $effect(() => {
    const c = content;
    if (view && c !== view.state.doc.toString()) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: c } });
    }
  });

  export function goToLine(n) {
    if (!view) return;
    const line = view.state.doc.line(Math.min(Math.max(n, 1), view.state.doc.lines));
    view.dispatch({ selection: { anchor: line.from }, effects: EditorView.scrollIntoView(line.from, { y: "start", yMargin: 24 }) });
    view.focus();
  }

  export function focus() {
    view?.focus();
  }
</script>

<div class="editor" bind:this={host}></div>

<style>
  .editor { height: 100%; min-height: 0; }
</style>
