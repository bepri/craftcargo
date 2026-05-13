---
marp: true
theme: default
paginate: true
backgroundColor: #1a1a2e
color: #eaeaea
style: |
  section {
    font-family: 'Segoe UI', sans-serif;
  }
  section.lead {
    text-align: center;
    justify-content: center;
  }
  h1 {
    color: #00d4aa;
  }
  h2 {
    color: #00d4aa;
  }
  strong {
    color: #ffcc00;
  }
  code {
    background: #16213e;
    color: #00d4aa;
  }
  table {
    font-size: 0.85em;
  }
  th {
    background: #16213e;
    color: #00d4aa;
  }
  blockquote {
    border-left: 4px solid #00d4aa;
    color: #aaa;
  }
  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1em;
  }
---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

![bg opacity:0.15](images/rust-crates-bg.png)
<!-- IMAGE NEEDED: Abstract dark background with faint Rust gear logos / crate boxes floating -->

# "What if packaging Rust for Ubuntu took **5 seconds** instead of **5 hours**?"

---

![bg right:35% opacity:0.8](images/file-explosion.png)
<!-- IMAGE NEEDED: Illustration of a single crate box exploding into 15+ scattered documents/files -->

# The Pain

### This is what it takes to ship ONE Rust crate to Ubuntu today:

- **15+ hand-written files** per crate
- `debian/control`, `debian/rules`, `debian/copyright`, changelog, patches…
- One typo → broken build → rejected upload → start over
- Ubuntu has **800+ Rust crates** to maintain. Each needs this.

> Nobody wants to do this. That's the problem.

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

![bg opacity:0.1 blur:2px](images/terminal-glow.png)
<!-- IMAGE NEEDED: Glowing terminal/command prompt on dark background, minimal and clean -->

# We built **craftcargo**.

```
$ craftcargo package-debcraft tokio-util
✓ debcraft.yaml generated
```

### One command. One file. Done.

---

![bg right:45%](images/demo-screenshot.png)
<!-- IMAGE NEEDED: Screenshot of the actual demo running in terminal - capture after building -->
<!-- Generate with: cd demo && ./run-demo.sh -n, then screenshot the YAML output -->

# Live Demo

### Key beats:

1. 🚀 Run the command (5 seconds)
2. 📄 Show the generated YAML (clean, readable)
3. ⚖️ Compare: **1 file** vs **15 files** from the old way

> The audience should feel the difference.

---

![bg left:30% opacity:0.9](images/architecture.png)
<!-- IMAGE NEEDED: Architecture diagram (use Excalidraw or Mermaid):
     [crates.io registry] → [craftcargo] → [debcraft.yaml] → [debcraft] → [.deb package]
     Hand-drawn style looks great for hackathons -->

# How We Did It

### Built on debcargo's battle-tested internals, reimagined the output

- Crate metadata → structured YAML (not scattered files)
- **SPDX licence normalisation** (handles complex expressions)
- Intelligent URL derivation (GitHub, GitLab, Salsa)
- Full Debian dependency resolution in one pass
- Schema matches debcraft's spec — plug and play

---

![bg opacity:0.08](images/graph-up.png)
<!-- IMAGE NEEDED: Subtle background of an upward-trending graph or green arrows -->

# The Numbers

| Metric | Before | **After** |
|--------|--------|-----------|
| Files to maintain per crate | 15+ | **1** |
| Time to package a new crate | ~2 hours | **5 seconds** |
| Human errors per packaging | Frequent | **Zero** (generated) |
| Crates we can scale to | Dozens (manual) | **Hundreds** (automated) |

---

![bg right:30%](images/ci-badges.png)
<!-- IMAGE NEEDED: Screenshot of green CI badges / GitHub Actions passing checks
     OR: a collage of ✅ checkmarks and test output -->

# Built to Last (Not Just a Hack)

### This isn't throwaway code:

- ✅ **174 automated tests** across every module
- ✅ CI pipeline — build, test, lint, format
- ✅ AI-powered code review on every PR
- ✅ Daily automated bug-hunting agent
- ✅ Clean Rust — no unsafe, no shortcuts

---

![bg right:40%](images/pipeline-vision.png)
<!-- IMAGE NEEDED: Funnel/pipeline diagram (Excalidraw style):
     Top: hundreds of crate boxes pouring in
     Middle: craftcargo machine/funnel
     Bottom: neat row of .deb packages coming out
     Style: colourful, hand-drawn, energetic -->

# The Vision

### The bridge between Rust's ecosystem and Ubuntu's

- **Today:** generate debcraft.yaml from any crate ✅
- **Next:** full build integration (yaml → .deb)
- **Future:** bulk packaging — 800 crates, 800 packages
- **End game:** Rust in Ubuntu stays fresh, automatically

---

![bg opacity:0.12](images/ubuntu-globe.png)
<!-- IMAGE NEEDED: Faint Ubuntu circle-of-friends logo or globe/earth image -->

# Why You Should Care

### A real problem for a real distro used by millions

- Ubuntu ships on servers, desktops, IoT, cloud
- Rust adoption is exploding — distros can't keep up manually
- craftcargo makes it **sustainable**
- Open source, built to upstream

**We didn't just hack on a toy. We built infrastructure.**

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

![bg opacity:0.1](images/rust-crates-bg.png)

# Try It

### `github.com/bepri/craftcargo`

```bash
cargo build --release
./target/release/debcargo package-debcraft <any-crate>
```

**Questions? Let's chat.**
