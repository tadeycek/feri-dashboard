// Markdown helpers for the notes workspace: rendering (reading view), frontmatter, outline, links.
import { marked } from "marked";
import DOMPurify from "dompurify";

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const unquote = (v) => v.trim().replace(/^"(.*)"$/, "$1").replace(/^'(.*)'$/, "$1");

/** Lower-cases and strips diacritics so "racunalnistvo" finds "računalništvo". */
export const fold = (s) => s.normalize("NFD").replace(/[\u0300-\u036f]/g, "").replace(/đ/g, "d").toLowerCase();

export const slug = (text) =>
  text.toLowerCase().replace(/<[^>]+>/g, "").replace(/[^\p{L}\p{N}\s-]/gu, "").trim().replace(/\s+/g, "-");

/** Splits `---` frontmatter off a note. `props` is [[key, string | string[]], …]. */
export function splitFrontmatter(text) {
  const m = /^---\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/.exec(text);
  if (!m) return { props: [], body: text, lines: 0 };
  const props = [];
  let cur = null;
  for (const line of m[1].split(/\r?\n/)) {
    const kv = /^([^\s:#][^:]*):\s*(.*)$/.exec(line);
    const item = /^\s+-\s*(.*)$/.exec(line);
    if (kv) {
      cur = [kv[1].trim(), unquote(kv[2])];
      props.push(cur);
    } else if (cur && item) {
      cur[1] = Array.isArray(cur[1]) ? [...cur[1], unquote(item[1])] : [unquote(item[1])];
    }
  }
  return { props, body: text.slice(m[0].length), lines: m[0].split("\n").length - 1 };
}

/** Headings with their 1-based line numbers in the full note (code fences and frontmatter skipped). */
export function outline(content) {
  const skip = splitFrontmatter(content).lines;
  const out = [];
  let fence = false;
  content.split("\n").forEach((line, i) => {
    if (i < skip) return;
    if (/^\s*(```|~~~)/.test(line)) { fence = !fence; return; }
    if (fence) return;
    const m = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(line);
    if (m && m[2]) out.push({ level: m[1].length, text: m[2], line: i + 1, id: slug(m[2]) });
  });
  return out;
}

/** Distinct note names linked with [[…]] (embeds excluded), without alias or heading. */
export function links(content) {
  const names = new Set();
  for (const m of content.matchAll(/(!?)\[\[([^\]\n]+)\]\]/g)) {
    if (m[1]) continue;
    names.add(m[2].split("|")[0].split("#")[0].trim());
  }
  names.delete("");
  return [...names];
}

export function wordStats(content) {
  const body = splitFrontmatter(content).body;
  const words = body.trim() ? body.trim().split(/\s+/).length : 0;
  return { words, chars: body.length };
}

/** Flips the n-th task checkbox (`- [ ]`) in source order. */
export function toggleTask(content, n) {
  const lines = content.split("\n");
  let fence = false;
  let idx = 0;
  for (let i = 0; i < lines.length; i++) {
    if (/^\s*(```|~~~)/.test(lines[i])) { fence = !fence; continue; }
    if (fence) continue;
    const m = /^(\s*(?:[-*+]|\d+[.)])\s+\[)([ xX])\]/.exec(lines[i]);
    if (!m) continue;
    if (idx === n) {
      lines[i] = m[1] + (m[2] === " " ? "x" : " ") + lines[i].slice(m[1].length + 1);
      return lines.join("\n");
    }
    idx++;
  }
  return content;
}

// ---------- rendering ----------

let resolver = () => true;

const wikilink = {
  name: "wikilink",
  level: "inline",
  start(src) {
    const i = src.search(/!?\[\[/);
    return i < 0 ? undefined : i;
  },
  tokenizer(src) {
    const m = /^(!?)\[\[([^\]\n]+?)\]\]/.exec(src);
    if (m) return { type: "wikilink", raw: m[0], embed: !!m[1], target: m[2] };
  },
  renderer(t) {
    const [left, alias] = t.target.split("|");
    const [name, heading] = left.split("#");
    const clean = name.trim();
    if (t.embed) {
      if (/\.base$/i.test(clean)) {
        return `<span class="embed-card" data-base="${esc(clean)}"><b>Table (Bases)</b><span>${esc(clean)}${heading ? " · " + esc(heading) : ""}</span><em>Obsidian renders this table. Open it there.</em></span>`;
      }
      if (/\.(png|jpe?g|gif|svg|webp|pdf|mp4|mp3)$/i.test(clean)) {
        return `<span class="embed-card"><b>Attachment</b><span>${esc(clean)}</span></span>`;
      }
      return `<a class="wikilink embed" data-link="${esc(clean)}" href="#">Embedded: ${esc(clean)}</a>`;
    }
    const ok = resolver(clean);
    const label = alias?.trim() || (heading ? `${clean} > ${heading}` : clean);
    return `<a class="wikilink${ok ? "" : " unresolved"}" data-link="${esc(clean)}" href="#">${esc(label)}</a>`;
  },
};

const CALLOUT = { info: "info", note: "info", abstract: "info", todo: "info", tip: "ok", success: "ok", check: "ok", done: "ok", question: "warn", warning: "warn", caution: "warn", danger: "bad", error: "bad", failure: "bad", bug: "bad", quote: "quote", example: "quote" };

marked.use({
  gfm: true,
  extensions: [wikilink],
  renderer: {
    heading({ tokens, depth, text }) {
      return `<h${depth} id="${esc(slug(text))}">${this.parser.parseInline(tokens)}</h${depth}>\n`;
    },
    checkbox({ checked }) {
      return `<input type="checkbox" class="task"${checked ? " checked" : ""}> `;
    },
    blockquote({ tokens }) {
      const inner = this.parser.parse(tokens);
      const m = /^<p>\[!(\w+)\]([+-])?([^\n<]*)(?:\n|<br>)?([\s\S]*?)<\/p>\n?/.exec(inner);
      if (!m) return `<blockquote>${inner}</blockquote>\n`;
      const type = m[1].toLowerCase();
      const title = m[3].trim() || type.charAt(0).toUpperCase() + type.slice(1);
      const first = m[4].trim() ? `<p>${m[4].trim()}</p>` : "";
      const rest = inner.slice(m[0].length);
      return `<div class="callout ${CALLOUT[type] ?? "info"}"><div class="callout-title">${title}</div><div class="callout-body">${first}${rest}</div></div>\n`;
    },
  },
});

/** Renders a note body (frontmatter already removed) to sanitized HTML. */
export function renderMarkdown(body, { resolve } = {}) {
  resolver = resolve ?? (() => true);
  const html = marked.parse(body, { async: false });
  return DOMPurify.sanitize(html, { ADD_ATTR: ["data-link", "data-base"], FORBID_TAGS: ["style", "form", "iframe", "object", "embed"] });
}
