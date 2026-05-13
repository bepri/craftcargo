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

# What if Ubuntu Rust packaging went from **112 files** to **1 YAML**?

### craftcargo turns any Rust crate into `debcraft.yaml`

---

![bg right:35% opacity:0.8](images/file-explosion.png)
<!-- IMAGE NEEDED: Illustration of a single crate box exploding into 112 scattered documents/files -->

# The Pain

### Packaging one crate the old way still explodes into a paper-cut factory

- Real demo: `tokio-util` → **112 files** from traditional `debcargo`
- `debian/control`, `rules`, copyright, patches, changelog…
- Huge review surface, huge maintenance cost
- Ubuntu/Debian need this to scale across hundreds of Rust crates

> Packaging should be infrastructure, not artisanal suffering.

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

![bg opacity:0.1 blur:2px](images/terminal-glow.png)
<!-- IMAGE NEEDED: Glowing terminal/command prompt on dark background, minimal and clean -->

# We built **craftcargo**.

```bash
$ craftcargo package-debcraft tokio-util
✓ debcraft.yaml generated
```

- Generates `debcraft.yaml` from **any Rust crate**
- One command, one descriptor, one thing to review
- Built on debcargo's battle-tested internals

---

![bg right:45%](images/demo-screenshot.png)
<!-- IMAGE NEEDED: Screenshot of the actual demo running in terminal - capture after building -->
<!-- Generate with: cd demo && ./run-demo.sh -n, then screenshot the YAML output -->

# Demo, Backed by Proof

- `tokio-util` demo result: **1 `debcraft.yaml`** vs **112 generated files**
- Judges can watch it live, or replay `demo/craftcargo-demo.cast`
- Backup recording is a crisp **59.6-second asciinema**

> Same crate. Same metadata. Wildly less packaging noise.

---

![bg left:30% opacity:0.9](images/architecture.png)
<!-- IMAGE NEEDED: Architecture diagram (use Excalidraw or Mermaid):
     [crates.io registry] → [craftcargo] → [debcraft.yaml] → [debcraft pack] → [.deb package]
     Hand-drawn style looks great for hackathons -->

# The Debcraft Story

- Generated YAML is designed for **`debcraft pack`**
- Flow: crate → `craftcargo` → `debcraft.yaml` → `.deb`
- The cargo plugin is still WIP upstream in debcraft
- The hand-off artefact is ready **today**

---

![bg opacity:0.08](images/graph-up.png)
<!-- IMAGE NEEDED: Subtle background of an upward-trending graph or green arrows -->

# More Than One Trick

- `package-debcraft` — generate `debcraft.yaml`
- `deb-src-name` — Debian source naming
- `package` — traditional debcargo packaging
- `build-order`, `deb-dependencies`, `update-dependencies`

### One tool for generation, naming, dependency insight, and migration

---

![bg right:30%](images/ci-badges.png)
<!-- IMAGE NEEDED: Screenshot of green CI badges / GitHub Actions passing checks
     OR: a collage of ✅ checkmarks and test output -->

# Built to Last

- ✅ **174 tests passing**
- ✅ CI: build, test, clippy, fmt
- ✅ Copilot-based PR reviews
- ✅ Daily bug-hunt agent
- ✅ Hackathon speed, production discipline

---

![bg right:40%](images/pipeline-vision.png)
<!-- IMAGE NEEDED: Funnel/pipeline diagram (Excalidraw style):
     Top: hundreds of crate boxes pouring in
     Middle: craftcargo machine/funnel
     Bottom: neat row of .deb packages coming out
     Style: colourful, hand-drawn, energetic -->

# Docs That Scale Too

- Documentation follows **Diátaxis**
- Tutorial, how-to, explanation, reference
- Published with **mdBook** on **GitHub Pages**
- Easy for contributors to learn, adopt, and customise

---

![bg opacity:0.12](images/ubuntu-globe.png)
<!-- IMAGE NEEDED: Faint Ubuntu circle-of-friends logo or globe/earth image -->

# Why This Matters

- **Today:** generate `debcraft.yaml` from any Rust crate ✅
- **Next:** tighten the `debcraft pack` path as upstream lands
- **Then:** package Rust for Ubuntu/Debian at ecosystem scale

**We didn't build a toy. We built distro infrastructure.**

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

![bg opacity:0.1](images/rust-crates-bg.png)

# Try It

### `github.com/bepri/craftcargo` • branch `experimental/hack`

```bash
cargo build --release
./target/release/debcargo package-debcraft <any-crate>
```

**Questions? Let's chat.**
