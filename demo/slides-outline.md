# craftcargo — Hackathon Presentation
# (Google Slides / PowerPoint)

---

## Slide 1: Hook

**"What if packaging Rust for Ubuntu took 5 seconds instead of 5 hours?"**

- Big bold text, dramatic
- No logos, no fluff — just the question
- *Visual: a single terminal prompt blinking*

---

## Slide 2: The Pain (Make Them Feel It)

**This is what it takes to ship ONE Rust crate to Ubuntu today:**

- 15+ hand-written files per crate
- debian/control, debian/rules, debian/copyright, changelog, patches…
- One typo = broken build, rejected upload, start over
- Ubuntu has 800+ Rust crates to maintain. EACH needs this.
- Update a dependency? Do it all again.

*Visual: wall of file listings, red error messages, chaos*

**Nobody wants to do this. That's the problem.**

---

## Slide 3: The Drop

**We built craftcargo.**

```
$ craftcargo package-debcraft tokio-util
✓ debcraft.yaml generated
```

**One command. One file. Done.**

*Visual: terminal output, clean and satisfying*

---

## Slide 4: Live Demo

**[SWITCH TO TERMINAL — RUN THE DEMO]**

Key beats:
1. Run the command (5 seconds)
2. Show the generated YAML (clean, readable)
3. Compare: 1 file vs 15 files from the old way
4. Audience should feel the difference viscerally

*If time is tight: pre-recorded terminal GIF as backup*

---

## Slide 5: How We Did It

**Built on debcargo's battle-tested internals, but reimagined the output**

- Crate metadata → structured YAML (not scattered files)
- SPDX licence normalisation (handles "MIT OR Apache-2.0 WITH LLVM-exception")
- Intelligent URL derivation (GitHub, GitLab, Salsa — auto-detected)
- Full Debian dependency resolution in one pass
- Schema matches debcraft's spec exactly — plug and play

*Visual: clean architecture diagram, 4 boxes with arrows*

---

## Slide 6: The Numbers

| Metric | Before | After |
|--------|--------|-------|
| Files to maintain per crate | 15+ | **1** |
| Time to package a new crate | ~2 hours | **5 seconds** |
| Human errors per packaging | Frequent | **Zero** (generated) |
| Crates we can scale to | Dozens (manual) | **Hundreds** (automated) |

*Visual: big bold numbers, green arrows going up/down*

---

## Slide 7: Built to Last (Not Just a Hack)

**This isn't throwaway code:**

- 174 automated tests across every module
- CI pipeline with build, test, lint, format checks
- AI-powered code review on every PR
- Daily automated bug-hunting agent that opens issues
- Clean Rust — no unsafe, no unwrap-happy shortcuts

*Visual: green CI badges, test count*

---

## Slide 8: What's Next (The Vision)

**craftcargo is the bridge between Rust's ecosystem and Ubuntu's**

- Today: generate debcraft.yaml from any crate on crates.io ✅
- Next: full build integration (yaml → .deb in one pipeline)
- Future: automated bulk packaging — feed it 800 crates, get 800 packages
- End game: Rust in Ubuntu stays fresh, automatically

*Visual: funnel diagram — crates.io → craftcargo → Ubuntu archive*

---

## Slide 9: Why You Should Care

**This solves a real problem for a real distro used by millions**

- Ubuntu ships on servers, desktops, IoT, cloud
- Rust adoption is exploding — distros can't keep up manually
- craftcargo makes it sustainable
- Open source, built to upstream

**We didn't just hack on a toy. We built infrastructure.**

---

## Slide 10: Try It

**github.com/bepri/craftcargo**

```
cargo build --release
./target/release/debcargo package-debcraft <any-crate>
```

*[Team names / handles]*

**Questions? Let's chat.**

---

## Presenter Notes

### Timing (aim for 3-5 min total):
- Slides 1-2: 30s (hook + pain)
- Slide 3: 10s (the drop — fast, punchy)
- Slide 4: 60-90s (live demo is the centrepiece)
- Slides 5-6: 30s (quick technical + numbers)
- Slides 7-9: 30s (credibility + vision)
- Slide 10: 10s (call to action)

### Demo tips:
- Pre-build the binary before presenting
- Have `pv` installed for the typing simulation
- Run `cd demo && ./run-demo.sh` — press ENTER to advance
- If anything breaks, have pre-generated output in a backup terminal
- The demo crate (tokio-util) was chosen because it's complex enough to be impressive

### If judges ask:
- "How is this different from debcargo?" — We BUILD ON debcargo's internals but
  produce a single declarative YAML instead of imperative debian/ files. It's the
  next generation.
- "Does it actually build packages?" — The YAML plugs directly into debcraft
  (Ubuntu's new build system). We generate the input; debcraft does the build.
- "Why Rust?" — Because Rust's crate ecosystem is massive, growing fast, and
  distros are drowning trying to keep up manually.
- "What was hardest?" — Getting the licence normalisation and dependency mapping
  right. Debian has strict rules and edge cases everywhere.
