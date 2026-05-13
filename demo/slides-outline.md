# craftcargo — Slide Deck Outline
# (For Google Slides / PowerPoint)

---

## Slide 1: Title

**craftcargo: Rust → Ubuntu Packaging, Simplified**

- Subtitle: Automating debcraft.yaml generation from Rust crates
- Team / Date / Logo

---

## Slide 2: The Problem

**Packaging Rust for Ubuntu is painful**

- Each crate needs ~15 hand-maintained Debian files
- debian/control, debian/rules, debian/copyright, patches, etc.
- Dependency graphs are complex (tokio alone has 30+ transitive deps)
- Manual process is error-prone and slow to update
- Existing tool (debcargo) helps but still produces many files

*Visual: diagram showing a crate exploding into 15 files*

---

## Slide 3: The Solution

**craftcargo: One YAML to rule them all**

- Generates a single `debcraft.yaml` from any Rust crate
- Maps Cargo.toml metadata → Debian packaging fields automatically
- Handles: licensing (SPDX → DEP-5), dependencies, features, multi-arch
- Output is human-readable, easy to review, version-controllable

*Visual: crate → craftcargo → debcraft.yaml (simple flow)*

---

## Slide 4: How It Works

**Under the hood**

1. Fetches crate metadata from crates.io registry
2. Resolves dependencies and maps to Debian package names
3. Normalises licences (SPDX → Debian format)
4. Generates VCS URLs, homepage, issues links
5. Produces structured YAML for debcraft consumption

*Visual: architecture diagram — registry → craftcargo → yaml → debcraft → .deb*

---

## Slide 5: Live Demo

**End-to-end: tokio-util → debcraft.yaml**

- [Run the demo script or show recorded terminal]
- Key moments:
  - Single command invocation
  - Generated YAML structure
  - Comparison: 1 file vs 15 files

---

## Slide 6: Key Features

| Feature | Status |
|---------|--------|
| Crate metadata extraction | ✅ Done |
| Debian naming conventions | ✅ Done |
| SPDX licence normalisation | ✅ Done |
| Dependency mapping | ✅ Done |
| VCS/URL derivation | ✅ Done |
| debcraft.yaml schema | ✅ Done |
| Package generation | ✅ Done |
| Parts/build steps | 🔧 In Progress |
| Companion files | 🔧 In Progress |
| Full debcraft build integration | 📋 Planned |

---

## Slide 7: Quality & CI

**Built with confidence**

- 174+ automated tests covering all modules
- CI pipeline: build, test, clippy, formatting
- Copilot code review on every PR
- Daily automated bug hunts (scheduled CI agent)
- Test coverage: naming, versions, dependencies, serialisation, URL logic

---

## Slide 8: What's Next

**Roadmap**

- Complete parts/build step generation (step 7)
- Companion file generation (step 8)
- Integration testing with real debcraft builds
- Upstream contribution to debcargo ecosystem
- Support for workspace crates (multi-crate repos)

---

## Slide 9: Impact

**Why this matters**

- Reduces packaging time from hours → minutes
- Fewer human errors in packaging metadata
- Easier for new contributors to package Rust for Ubuntu
- Scales to hundreds of crates in the Ubuntu Rust ecosystem
- Single source of truth (YAML) instead of scattered files

---

## Slide 10: Questions & Demo

**Thank you!**

- Repository: github.com/bepri/craftcargo
- Try it: `debcargo package-debcraft <crate-name>`
- Questions?

---

## Speaker Notes

### For Slide 5 (Demo):
- Build the project first: `cargo build --release`
- Run: `cd demo && ./run-demo.sh`
- If time is short, use `./run-demo.sh -n` for auto-advance
- Have a terminal ready with the output pre-generated as backup

### For Slide 6 (Features):
- Emphasise that the core pipeline is working end-to-end
- "In Progress" items have the architecture in place, just need logic filled in
- The todo!() stubs are clearly defined and being worked on by the team

### For Q&A:
- "How does this compare to debcargo?" — It builds ON TOP of debcargo, reusing its
  registry logic and dependency resolution, but outputs debcraft.yaml instead of
  raw debian/ files
- "What about non-crates.io crates?" — Currently focused on registry crates;
  git-source support is planned
- "Can debcraft actually build from this YAML?" — That's the debcraft side;
  craftcargo produces the input, debcraft consumes it
