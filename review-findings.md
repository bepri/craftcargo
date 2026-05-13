# craftcargo README — Review Findings
Generated from three parallel subagent reviews of `/home/ubuntu/work/craftcargo/README.md`.

---

## Consensus (flagged by multiple reviewers)

1. **`$CRAFT_PROJECT_VERSION` in `source-tag` is broken** (YAML + Debian reviewers)
   - Expands to full Debian version e.g. `14.1.0-1`; upstream git tags don't have the `-1` suffix.
   - Library crate variant: `v$CRAFT_PROJECT_VERSION` → `v1.0.197-1`, actual tag is `v1.0.197`.
   - Fix: craftcargo must strip the Debian revision and emit a literal upstream version string.
   - Open question: does debcraft expose a `$CRAFT_UPSTREAM_VERSION` variable, or must it always be a literal?

2. **Library crate packaging is fundamentally wrong** (YAML + Debian reviewers)
   - `plugin: rust` runs `cargo install`, which requires a binary target — fails on library-only crates.
   - `multi-arch: foreign` should be `multi-arch: same` (Debian Rust Team policy).
   - `section: libdevel` should be `section: rust`.
   - `architecture: all` is missing.
   - Cargo feature `provides:` entries entirely absent (e.g. `librust-serde+derive-dev`).
   - `.cargo-checksum.json` not mentioned; required by Cargo for the registry directory to be valid.
   - Fix: library crates need `plugin: nil` + `override-build` that copies source tree into
     `$CRAFT_PART_INSTALL/usr/share/cargo/registry/<crate>-<version>/`.

3. **`debian/changelog` reference is wrong** (Debian + docs reviewers)
   - debcraft doesn't use `debian/changelog`; the version lives in the YAML `version:` field.
   - Sentence to fix: "update `debian/changelog` as you normally would before handing off to debcraft."

---

## YAML Accuracy Reviewer Findings

### Critical
- **`organize:` path `(build)/target/release/` is likely wrong**
  - The rust plugin runs `cargo install --root $CRAFT_PART_INSTALL`, placing binaries at
    `bin/<name>` inside the install dir, not in the build dir.
  - Correct organize entry is probably `bin/rg: usr/bin/rg`, not `(build)/target/release/rg: usr/bin/rg`.
  - Caveat: if debcraft's rust plugin diverges from snapcraft's and does NOT run `cargo install`,
    then `(build)/target/release/` is correct — but this needs to be confirmed and documented.

- **Multi-binary example has no per-package file routing**
  - Both binaries land in the shared staging area with no `stage:` filters or equivalent.
  - Without routing, both packages contain both binaries → `dpkg` conflict on install.
  - Needs per-package file assignment (debcraft equivalent of craft-parts `stage:` patterns).

### Medium
- **`rust-channel: none` is not a valid plugin value**
  - The real approach for using system rustc/cargo: omit `rust-channel`, add `rustc`/`cargo` to
    `build-packages`, and possibly set `rust-ignore-toolchain-file: true`.
  - Remove the "Set to `none`..." sentence from the plugin key table.

- **Vendoring YAML is structurally identical to the non-vendored case**
  - This is correct (the rust plugin reads `.cargo/config.toml` automatically), but it reads
    as a mistake. Add a clear note that this is intentional — no YAML change is required.

- **`$CRAFT_PROJECT_VERSION` may be debcraft-specific, not a stock craft-parts variable**
  - Standard craft-parts exposes `$CRAFT_PART_INSTALL`, `$CRAFT_PART_BUILD`, etc.
  - `$CRAFT_PROJECT_VERSION` needs to be confirmed as a debcraft injection and noted as such.

### Low
- `architecture: all` missing from library package (`multi-arch: same` only makes sense with it).
- `section: utils` emitted redundantly at both top-level and package-level; omit if identical.
- `# rust-features: []` is a no-op placeholder; use `# rust-features: [feature-name]` or omit.
- Library `organize:` hardcodes `serde-1.0.197`; craft-parts has no variable interpolation in
  organize values — `craftcargo update` must regenerate this as a literal on each version bump.
  Document this as intentional.

---

## Debian Packaging Reviewer Findings

### High
- **Cargo feature `provides:` entirely absent**
  - Real `librust-*-dev` packages provide feature virtual packages:
    `librust-serde+derive-dev`, `librust-serde+std-dev`, `librust-serde+default-dev`, etc.
  - Without these, downstream crates that depend on specific features fail to build.
  - Also need versioned forms: `librust-serde-1+derive-dev`, `librust-serde-1.0+derive-dev`, etc.
  - May be impractical to auto-generate fully; at minimum document as a known limitation.

- **`.cargo-checksum.json` not mentioned**
  - Required alongside crate source in `usr/share/cargo/registry/<crate>-<version>/`.
  - Cargo rejects the directory as an invalid registry entry without it.

### Medium
- **`provides:` entries should be versioned**
  - debcargo practice: `librust-serde-1-dev (= 1.0.197-1)`, not bare virtual package names.
  - Unversioned form technically works but weakens apt co-installability reasoning.

- **`section: libdevel` → `section: rust`**
  - Every Rust library package in the Debian/Ubuntu archive uses `section: rust`
    (shown as `universe/rust`). `libdevel` would trigger a lintian warning.

- **`debian/changelog`** (see Consensus #3 above)

- **`debian/copyright` location unexplained**
  - The known limitation says "maintain it by hand", but doesn't say where the file should
    live in a debcraft project (no `debian/` directory). Needs clarification.

### Low
- **Comparison table: debcargo "Local workspace: No" is inaccurate**
  - debcargo accepts `--directory` for local crates. The real limitation is workspace support.
  - Suggested wording: debcargo "Single local crate (no workspace)" vs craftcargo "Full workspace (root crate)".

- **`--lib` flag should clarify binary suppression**
  - When `--lib` is active, does craftcargo suppress binary packages or generate both?
  - debcargo can emit both `<crate>` and `librust-<crate>-dev` from the same source; craftcargo
    should document its behaviour.

---

## Docs Quality Reviewer Findings

### High
- **`rustc` in prerequisites table is misleading**
  - "Required at build time by debcraft" — but the rust plugin manages its own toolchain.
  - A maintainer will install `rustc` from the archive, then be confused when debcraft
    downloads a different version. Remove or reword: "debcraft's rust plugin manages its
    own toolchain; no system Rust installation is required."

- **`craftcargo.toml` schema is internally inconsistent**
  - `[package]` (singular, no key) vs `[packages.ripgrep]` (plural, keyed) — relationship undefined.
  - Does `[package]` apply globally? Can it coexist with `[packages.*]` in multi-binary cases?
  - `[dep-map]` uses a different naming style from all other sections (no nesting, kebab-case).
  - Needs either a schema explanation or a normalisation to one style.

- **Override merge semantics underspecified**
  - "keys take precedence" doesn't explain deep merge vs. table replacement.
  - What happens when a `craftcargo.toml` key refers to a dependency the new upstream dropped?
  - What happens to manually-added `build-packages` entries after `craftcargo update`?

- **Several reader questions go unanswered**
  - What happens if `Cargo.toml` has no `repository` field? (emit `# FIXME`? fail? skip?)
  - Does `craftcargo generate` overwrite an existing `debcraft.yaml` silently or prompt?
  - How do you package a non-root workspace member? (is `craftcargo generate ./member` supported?)

### Medium
- **Known limitations mixes temporary and permanent constraints**
  - "Not yet implemented" items vs. permanent design limits should be in separate lists.

- **Topic sections (vendoring, multi-binary, library) have no command examples**
  - After editing `craftcargo.toml`, readers don't know they need `craftcargo update` to regenerate.

- **`build-packages` / `build-depends` used without definition**
  - New-to-debcraft maintainers won't know what `build-packages` maps to in the .deb build system.

### Low
- Tone inconsistency: introduction uses marketing language; body is dry reference.
- `[dep-map]` naming style differs from all other `craftcargo.toml` sections.
- "Updating an existing descriptor" partially duplicates the `craftcargo update` command entry.

---

## Items confirmed correct by reviewers

- Top-level YAML field names and positions (`name`, `version`, `summary`, `description`, `base`,
  `maintainer`, `section`, `license`) ✅
- `base: ubuntu@24.04` format ✅
- All rust plugin key names in the reference table ✅
- `build-packages: [libpcre2-dev]` (and omitting `rustc`/`cargo`) ✅
- ripgrep binary named `rg`, not `ripgrep` ✅
- SPDX `license: MIT OR Unlicense` ✅
- `librust-*-dev` naming convention ✅
- `provides:` with SemVer-decomposed virtual packages (structure correct; versioning and
  features need fixing) ✅/⚠️
- `usr/share/cargo/registry/serde-1.0.197` install path (per Debian Rust Team policy) ✅
- `(build)/` prefix syntax in organize is valid craft-parts ✅
