# AGENTS.md

Instructions for AI coding agents working on this repository.

## Project Overview

**craftcargo** (binary: `debcargo`) is a tool that generates `debcraft.yaml`
descriptor files from Rust crates for Ubuntu/Debian packaging. It builds on
top of the upstream debcargo tool, adding the `package-debcraft` subcommand
that produces a single YAML file consumable by [debcraft](https://launchpad.net/debcraft).

## Language and Style

- All prose (comments, documentation, commit messages) must use **South African English**:
  - "customise" not "customize"
  - "normalise" not "normalize"
  - "licence" (noun) not "license"
  - "colour" not "color"
  - "organisation" not "organization"
- **Code identifiers are exempt** — do not rename `Serialize`, `Deserialize`,
  `normalize_spdx_license`, `skip_serializing_if`, or similar Rust/serde identifiers.
- The `organize:` key in YAML output is a debcraft schema field — leave it unchanged.

## Architecture

```
src/
├── cli.rs              — CLI argument parsing (clap derive)
├── lib.rs              — Library root, re-exports
├── package.rs          — Traditional debian/ packaging logic
├── debcraft/
│   ├── mod.rs          — debcraft.yaml generation (package-debcraft subcommand)
│   └── schema.rs       — Serde structs for debcraft.yaml format
├── debian/             — Debian control file generation helpers
├── crates.rs           — Crates.io index interaction
├── config.rs           — Configuration file handling
└── util.rs             — Shared utilities
```

## Building and Testing

```bash
cargo build             # Debug build
cargo build --release   # Release build
cargo test              # Run all 174+ tests
cargo clippy            # Lint checks
cargo fmt -- --check    # Format check
```

The binary is `target/{debug,release}/debcargo`. Key subcommands:
- `debcargo package-debcraft <crate>` — generates debcraft.yaml (our addition)
- `debcargo deb-src-name <crate>` — prints Debian source package name
- `debcargo package <crate>` — traditional debian/ directory generation

## Important Conventions

- **Cross-device links**: debcargo uses hard links internally. Temporary
  directories must be on the same filesystem as the working directory (never
  use `/tmp` if it's a separate mount).
- **DEBFULLNAME/DEBEMAIL**: The `package` subcommand requires these environment
  variables to be set. `package-debcraft` does not.
- **Tests**: Located alongside source in `src/` (unit tests) and in `tests/`
  (integration/expected-output tests). Use `cargo test` to run all.

## Documentation

Documentation lives in `book/` (mdBook format) and follows the
[Diátaxis framework](https://diataxis.fr/):
- `book/src/tutorial/` — Learning-oriented guides
- `book/src/how-to/` — Task-oriented recipes
- `book/src/reference/` — Information-oriented specs
- `book/src/explanation/` — Understanding-oriented discussion

Build docs with: `mdbook build book/`

## CI Workflows

- `.github/workflows/ci.yml` — Build, test, clippy, fmt on PRs and pushes
- `.github/workflows/copilot-review.yml` — AI code review on PRs
- `.github/workflows/bug-hunt.yml` — Daily scheduled: tests, clippy, audit, unsafe scan
- `.github/workflows/docs.yml` — mdBook → GitHub Pages deployment

## Demo

The `demo/` directory contains:
- `run-demo.sh` — Interactive demo using demo-magic (uses `bon` crate)
- `craftcargo-demo.cast` — Pre-recorded asciinema cast
- `slides.md` — Marp presentation (export with `marp slides.md --pptx`)
- `record-cast.sh` — Script to re-record the asciinema cast

Run the demo: `cd demo && bash run-demo.sh -d -n` (non-interactive mode)

## Working on This Repo

When making changes:
1. Ensure `cargo test` passes before committing
2. Run `cargo clippy` and address any warnings
3. Keep documentation in sync with code changes
4. Use descriptive commit messages in SA English
5. Include `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>` in commits

## Known Limitations

- The `debcraft pack` command does not yet support package names containing `+`
  (Rust feature packages like `librust-foo+bar-dev`). The `bon` crate works
  because its features map to separate named packages without `+` in the
  package name itself.
- Three `todo!()` stubs remain in `src/debcraft/mod.rs`:
  `build_debcraft_packages()`, `build_debcraft_parts()`, `write_companion_files()`
