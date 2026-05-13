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
---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

# "What if packaging Rust for Ubuntu took **5 seconds** instead of **5 hours**?"

---

# The Pain

### This is what it takes to ship ONE Rust crate to Ubuntu today:

- **15+ hand-written files** per crate
- `debian/control`, `debian/rules`, `debian/copyright`, changelog, patches…
- One typo → broken build → rejected upload → start over
- Ubuntu has **800+ Rust crates** to maintain. Each needs this.
- Update a dependency? Do it all again.

> Nobody wants to do this. That's the problem.

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

# We built **craftcargo**.

```
$ craftcargo package-debcraft tokio-util
✓ debcraft.yaml generated
```

### One command. One file. Done.

---

# Live Demo

<!-- Switch to terminal and run: cd demo && ./run-demo.sh -->

### Key beats:

1. 🚀 Run the command (5 seconds)
2. 📄 Show the generated YAML (clean, readable)
3. ⚖️ Compare: **1 file** vs **15 files** from the old way

> The audience should feel the difference.

---

# How We Did It

### Built on debcargo's battle-tested internals, reimagined the output

- Crate metadata → structured YAML (not scattered files)
- **SPDX licence normalisation** (handles `MIT OR Apache-2.0 WITH LLVM-exception`)
- Intelligent URL derivation (GitHub, GitLab, Salsa — auto-detected)
- Full Debian dependency resolution in one pass
- Schema matches debcraft's spec exactly — plug and play

---

# The Numbers

| Metric | Before | **After** |
|--------|--------|-----------|
| Files to maintain per crate | 15+ | **1** |
| Time to package a new crate | ~2 hours | **5 seconds** |
| Human errors per packaging | Frequent | **Zero** (generated) |
| Crates we can scale to | Dozens (manual) | **Hundreds** (automated) |

---

# Built to Last (Not Just a Hack)

### This isn't throwaway code:

- ✅ **174 automated tests** across every module
- ✅ CI pipeline — build, test, lint, format
- ✅ AI-powered code review on every PR
- ✅ Daily automated bug-hunting agent that opens issues
- ✅ Clean Rust — no unsafe, no unwrap-happy shortcuts

---

# The Vision

### craftcargo is the bridge between Rust's ecosystem and Ubuntu's

- **Today:** generate debcraft.yaml from any crate on crates.io ✅
- **Next:** full build integration (yaml → .deb in one pipeline)
- **Future:** automated bulk packaging — 800 crates, 800 packages
- **End game:** Rust in Ubuntu stays fresh, automatically

---

# Why You Should Care

### This solves a real problem for a real distro used by millions

- Ubuntu ships on servers, desktops, IoT, cloud
- Rust adoption is exploding — distros can't keep up manually
- craftcargo makes it **sustainable**
- Open source, built to upstream

**We didn't just hack on a toy. We built infrastructure.**

---

<!-- _class: lead -->
<!-- _backgroundColor: #0f0f23 -->

# Try It

### `github.com/bepri/craftcargo`

```bash
cargo build --release
./target/release/debcargo package-debcraft <any-crate>
```

**Questions? Let's chat.**
