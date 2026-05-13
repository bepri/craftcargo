# Slide Images

This directory holds images for the Marp presentation (`slides.md`).

## What's needed

| Slide | Filename | Suggestion |
|-------|----------|------------|
| 1 (Hook) | `rust-crates-bg.png` | Abstract dark bg with faint Rust gear logos / crate boxes. Try Unsplash "rust metal dark" or generate with AI. |
| 2 (Pain) | `file-explosion.png` | Illustration: single crate box exploding into 15+ scattered files. Excalidraw hand-drawn style works great. |
| 3 (The Drop) | `terminal-glow.png` | Minimal glowing terminal prompt on dark background. Unsplash "terminal hacker dark". |
| 4 (Demo) | `demo-screenshot.png` | **Capture this yourself:** run `cd demo && ./run-demo.sh -n` and screenshot the YAML output. |
| 5 (How) | `architecture.png` | Pipeline diagram: crates.io → craftcargo → debcraft.yaml → debcraft → .deb. See `architecture.mmd` for Mermaid source. |
| 6 (Numbers) | `graph-up.png` | Subtle upward-trending graph or green arrows. Unsplash "growth chart minimal". |
| 7 (Quality) | `ci-badges.png` | Screenshot of GitHub Actions with green ✅ checks passing. Take from the repo's Actions tab. |
| 8 (Vision) | `pipeline-vision.png` | Funnel diagram: many crate boxes → craftcargo machine → neat .deb packages. Excalidraw. |
| 9 (Why Care) | `ubuntu-globe.png` | Faint Ubuntu circle-of-friends logo or earth/globe. Ubuntu brand assets or Unsplash "earth minimal dark". |

## Quick generation options

1. **Excalidraw** (excalidraw.com) — hand-drawn diagrams, export as PNG with transparent bg
2. **Mermaid** — render `architecture.mmd` at mermaid.live, export dark-themed PNG
3. **Unsplash** — free stock photos (check licence: all Unsplash are free for commercial use)
4. **AI image gen** — DALL-E / Midjourney for the abstract backgrounds
5. **Screenshots** — terminal demo + CI badges from your actual repo

## Rendering the Mermaid diagram

```bash
# Option 1: mermaid-cli
npx -p @mermaid-js/mermaid-cli mmdc -i architecture.mmd -o architecture.png -t dark -b transparent

# Option 2: paste into mermaid.live and export
```
