# FERI Dashboard: design decisions

Written 2026-10-06 after checking the app against four articles on spotting vibe-coded frontends. Keep new screens consistent with this file. If a screen breaks a rule here, fix the screen.

## Decided by the owner (do not change without asking)
- **Colours:** Obsidian graphite surfaces plus one purple accent, light and dark. Five muted subject colours at equal lightness (OKLCH). Tokens are in `linux/src/app.css`.
- **Notes page:** a copy of Obsidian's layout (explorer, tabs, editor, right sidebar). The owner likes it as it is.
- **Language:** the interface is English. Vault content (note names, `Roki/`, `Predmeti/`, template headings) stays as the owner wrote it.

## Rules (from the research)
1. **No coloured side stripes** on cards, blocks, list rows or nav items. Subject identity is a small square, or a tinted block with the subject colour in the title.
2. **No hero metric.** No huge number with a small label above it. State facts in a sentence.
3. **Plain sections, not boxed cards.** A heading with a rule under it, content below. A box is only for a thing that is itself a grid (the timetable).
4. **One radius scale:** 4 px for controls, 6 px for blocks. No pills, except Obsidian's own property chips on the notes page.
5. **No labels above headings,** no icon tiles, no badge pills, no decorative glows or gradients.
6. **Text:** nothing under 13 px. Muted text must reach 4.5:1 (dark `#9a9a9a`, light `#6b6b75`).
7. **Motion:** only for state changes (dialog fade, refresh spinner). No entrance animations, no bounce, no pulsing dots.
8. **Type:** Barlow Condensed for dates, times and headings; Hanken Grotesk for text. Never Inter as the default.
9. **Copy:** say what a thing is or does. No marketing words, no dash in every sentence.
10. **Vary spacing:** related items close (6–10 px), groups apart (30–48 px).

## Audit on 2026-10-06 (before the redesign)
Matched the articles and were removed: coloured side stripes (week blocks, hero, timeline, subject bars, active nav item, callouts), hero metric countdown with a label above it, six identical boxed cards with one radius and one gap, pill badges, muted text at 3.4–3.7:1, text at 10–12.5 px, a "heading plus small button" repeated in every card.
Did not match: Inter, purple-to-blue gradients, glass or glow, emoji icons, pulsing dots, bouncy easing.
Known remaining risk: a purple accent. It is a single solid colour (Obsidian's), not a gradient, and the owner chose to keep it.

## Sources (read 2026-10-06)
- The visible tells of AI design: https://www.impeccable.style/slop
- AI Design Slop, overused UI patterns: https://noqta.tn/en/blog/ai-design-slop-overused-ui-patterns-fix-2026
- How to avoid that vibe coded look: https://robotsonpayroll.substack.com/p/how-to-avoid-that-vibe-coded-look
- Why vibe-coded websites look the same: https://codemyspec.com/blog/vibe-coded-websites-look-the-same
- Caveat from the searches: no visual check can prove a site was vibe coded, and polished AI-assisted work can look fully custom.
