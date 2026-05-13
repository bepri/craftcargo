# craftcargo

**craftcargo** is a command-line tool for Debian/Ubuntu package maintainers that automates
the creation of [`debcraft.yaml`][debcraft] build descriptors for Rust programs. It reads
a crate's `Cargo.toml` — either from a local workspace or directly from
[crates.io](https://crates.io) — and produces a ready-to-use `debcraft.yaml` that encodes
all the information debcraft needs to build and package the program.

Think of craftcargo as doing for debcraft what [debcargo][debcargo] does for classic Debian
source packages: it bridges the gap between the Cargo ecosystem and the Debian packaging
workflow, so you spend your time on policy compliance and changelog entries rather than on
boilerplate YAML.

---

## Table of Contents

1. [How it fits into the workflow](#how-it-fits-into-the-workflow)
2. [Installation](#installation)
3. [Prerequisites](#prerequisites)
4. [Quick start](#quick-start)
5. [Command reference](#command-reference)
6. [The generated debcraft.yaml](#the-generated-debcraftyaml)
7. [Overrides and the craftcargo.toml config file](#overrides-and-the-craftcargotoml-config-file)
8. [Dependency mapping](#dependency-mapping)
9. [Vendoring dependencies](#vendoring-dependencies)
10. [Multi-binary crates](#multi-binary-crates)
11. [Library crates](#library-crates)
12. [Updating an existing descriptor](#updating-an-existing-descriptor)
13. [Comparison with debcargo](#comparison-with-debcargo)
14. [Known limitations](#known-limitations)

---

## How it fits into the workflow

```
crates.io / local Cargo.toml
          │
          ▼
    craftcargo generate          ← you are here
          │
          ▼
     debcraft.yaml  ──────────────────────────────────┐
          │                                           │
          ▼                                           │
   debcraft build               optional: hand-edit  │
          │                     craftcargo.toml ──────┘
          ▼
  .deb / source package
```

craftcargo is a *one-shot generator*. It produces a `debcraft.yaml` and, optionally, a
`craftcargo.toml` override file that you keep in version control alongside any other
per-package customisations. When the upstream crate releases a new version you run
`craftcargo update` to refresh the descriptor while preserving your overrides.

---

## Installation

### From the Ubuntu archive (recommended)

```
sudo apt install craftcargo
```

### From source

```
git clone https://github.com/example/craftcargo
cd craftcargo
cargo build --release
sudo install -m 755 target/release/craftcargo /usr/local/bin/
```

---

## Prerequisites

| Tool | Purpose |
|------|---------|
| `debcraft` | Consumes the generated YAML to build packages |
| `cargo` | Inspects local workspaces and resolves `Cargo.lock` |
| `rustc` | Required at build time by debcraft when building the package |
| `lintian` | Optional; craftcargo can invoke it to pre-check the descriptor |

craftcargo does **not** require network access when working with a local crate. Network
access is only needed when fetching metadata from crates.io via `craftcargo generate
<crate> <version>`.

---

## Quick start

### Package a crate from crates.io

```sh
# Create a working directory and generate the descriptor
mkdir -p ~/packaging/ripgrep && cd ~/packaging/ripgrep
craftcargo generate ripgrep 14.1.0

# Review what was generated
cat debcraft.yaml

# Build the package
debcraft build
```

### Package a crate from a local workspace

```sh
cd ~/src/my-rust-tool
craftcargo generate .

cat debcraft.yaml
debcraft build
```

---

## Command reference

### `craftcargo generate`

```
craftcargo generate [OPTIONS] <CRATE|PATH> [VERSION]
```

Generates a fresh `debcraft.yaml` (and optionally a `craftcargo.toml` stub) in the current
directory.

**Arguments**

| Argument | Description |
|----------|-------------|
| `CRATE` | Crate name as published on crates.io |
| `PATH` | Path to a local crate root (must contain `Cargo.toml`). Use `.` for the current directory. |
| `VERSION` | Crate version to target. Required when `CRATE` is a crates.io name; ignored for `PATH`. |

**Options**

| Flag | Default | Description |
|------|---------|-------------|
| `--output <FILE>` | `debcraft.yaml` | Write the descriptor to `FILE` instead of the default name. |
| `--vendor` | off | Vendor all Cargo dependencies into a `vendor/` subdirectory and configure debcraft to use it offline. |
| `--lib` | auto-detected | Treat the crate as a library package even if it also exposes binaries. |
| `--base <BASE>` | host series | Target Ubuntu base in `ubuntu@YY.MM` format (e.g. `ubuntu@24.04`). Friendly series names (`noble`, `plucky`) are also accepted and converted automatically. |
| `--maintainer <NAME <EMAIL>>` | `$DEBEMAIL` / `$DEBFULLNAME` | Override the maintainer field. |
| `--no-config-stub` | off | Skip writing a `craftcargo.toml` override file. |
| `--lintian` | off | Run lintian on the generated descriptor and print warnings. |

**Examples**

```sh
# Target a specific Ubuntu base
craftcargo generate fd-find 10.2.0 --base ubuntu@24.04

# Vendor dependencies for air-gapped builds
craftcargo generate . --vendor

# Write to a custom path (useful in scripts)
craftcargo generate bat 0.24.0 --output packaging/bat/debcraft.yaml
```

---

### `craftcargo update`

```
craftcargo update [OPTIONS] [VERSION]
```

Re-generates the `debcraft.yaml` in the current directory for a new upstream version,
merging changes with any overrides recorded in `craftcargo.toml`. Safe to run repeatedly;
it will never discard hand-edited override keys.

**Arguments**

| Argument | Description |
|----------|-------------|
| `VERSION` | New upstream version. Omit to fetch the latest published version from crates.io. |

**Options**

| Flag | Description |
|------|-------------|
| `--dry-run` | Print the updated descriptor to stdout without writing it. |
| `--diff` | Show a unified diff of what would change. |

**Example**

```sh
# Bump to a specific version and preview changes
craftcargo update 14.2.0 --diff
```

---

### `craftcargo info`

```
craftcargo info <CRATE> [VERSION]
```

Prints a summary of the crate metadata that craftcargo would use when generating a
descriptor, without writing any files. Useful for investigating dependency trees before
committing to a packaging effort.

```sh
craftcargo info ripgrep 14.1.0
```

---

### `craftcargo check`

```
craftcargo check [OPTIONS]
```

Validates the `debcraft.yaml` and `craftcargo.toml` in the current directory for common
problems: missing mandatory fields, unknown override keys, dependency names that do not
exist in the Ubuntu archive, etc.

```sh
craftcargo check --base ubuntu@24.04
```

---

## The generated debcraft.yaml

Below is an annotated example of the file craftcargo produces for a binary crate. Fields
marked `# generated` are set automatically; fields marked `# review` are best-guess values
that a maintainer should verify.

```yaml
# debcraft.yaml – generated by craftcargo 0.1.0 for ripgrep 14.1.0

name: ripgrep                            # generated – matches crate name
version: 14.1.0-1                        # generated – upstream + Debian revision
summary: recursively search directories for a regex pattern  # generated from Cargo.toml
description: |                           # generated from Cargo.toml description
  ripgrep is a line-oriented search tool that recursively searches the current
  directory for a regex pattern. By default, ripgrep will respect gitignore
  rules and automatically skip hidden files/directories and binary files.
base: ubuntu@24.04                       # generated – from --base flag or host series
maintainer: Alice Example <alice@example.com>   # generated from $DEBEMAIL
section: utils                           # review  – craftcargo picks a best guess
license: MIT OR Unlicense                # generated – from Cargo.toml license field (SPDX)

packages:
  ripgrep:                               # generated – one entry per installable binary
    section: utils                       # inherits top-level value; override if needed

parts:
  ripgrep:                               # generated
    plugin: rust
    source: https://github.com/BurntSushi/ripgrep   # generated from Cargo.toml
    source-type: git
    source-tag: $CRAFT_PROJECT_VERSION
    # rust-channel: stable              # review – pin a specific channel or version if needed
    # rust-features: []                 # review – add non-default Cargo features if needed
    build-packages:                      # generated – sys-crate mappings only;
      - libpcre2-dev                     #   rustc/cargo are managed by the rust plugin
    organize:                            # generated – maps build output to install paths
      (build)/target/release/rg: usr/bin/rg
```

### Key sections

**Top-level fields** (`name`, `version`, `summary`, `description`, `base`, `maintainer`,
`section`, `license`) — package-wide metadata written directly at the top of the file.
`summary` comes from the `description` field in `Cargo.toml`; `description` comes from the
crate's README or long description, if available. `license` is populated from the `license`
field in `Cargo.toml` as an [SPDX 2.1 expression](https://spdx.org/licenses/) — it records
the project's overall license for display purposes. It is **not** a substitute for
`debian/copyright`: that file maps individual source paths to copyright holders and years
and is still required for Debian policy compliance.

**`packages`** — a map of binary package names to their per-package overrides. Each key is
a Debian package name. Supported fields include `summary`, `description`, `section`,
`depends`, `recommends`, `suggests`, `provides`, `breaks`, `replaces`, `conflicts`, and
`multi-arch`. Binary crates produce one entry per `[[bin]]` target; library crates produce
a `librust-<crate-name>-dev` entry.

**`parts`** — a map of named build steps. Each part specifies `plugin: rust`, which invokes
Cargo and manages the Rust toolchain automatically (no need to list `rustc` or `cargo` in
`build-packages`). The rust plugin accepts several plugin-specific keys that craftcargo
will populate from `Cargo.toml` where possible:

| Key | Type | Description |
|-----|------|-------------|
| `rust-channel` | string | Rust release channel or version (`stable`, `beta`, `nightly`, or a version number). Set to `none` to use `rustc`/`cargo` from `build-packages` instead. |
| `rust-features` | list | Cargo features to enable at build time (equivalent to `--features`). |
| `rust-no-default-features` | bool | If `true`, suppresses the crate's default features. |
| `rust-path` | list | Path(s) to the `Cargo.toml` to build. Defaults to `.`. |
| `rust-cargo-parameters` | list | Extra arguments appended to the `cargo` invocation. |
| `rust-use-global-lto` | bool | Enable fat LTO across all crates. Reduces binary size at the cost of build time. |
| `rust-ignore-toolchain-file` | bool | Ignore a `rust-toolchain.toml` file in the source tree. |

The `build-packages` list is generated by inspecting `[build-dependencies]` in `Cargo.toml`
and running `cargo metadata` to detect `*-sys` crates that proxy system libraries; each is
resolved via the bundled mapping table (see [Dependency mapping](#dependency-mapping)). The
`organize` map moves build artefacts from the part's staging area to their final install
paths.

---

## Overrides and the craftcargo.toml config file

`craftcargo.toml` is an optional, human-maintained companion file. When `craftcargo update`
is run, keys in this file take precedence over anything craftcargo would generate.

```toml
# craftcargo.toml

[package]
section = "text"          # override the auto-detected section
maintainer = "Bob <bob@ubuntu.com>"

[parts.ripgrep]
rust-features = ["pcre2"]  # enable the pcre2 Cargo feature at build time

[packages.ripgrep]
description = """
ripgrep (rg) is an extremely fast text search tool written in Rust.

This package was built with PCRE2 support enabled.
"""

[parts.ripgrep.organize]
# add any extra install paths that craftcargo did not detect automatically
"(build)/target/release/rg.bash-completion" = "usr/share/bash-completion/completions/rg"
```

> **Note:** lintian overrides are not yet supported in `debcraft.yaml`. Add them to
> `debian/source/lintian-overrides` by hand until debcraft gains native support.

All keys are optional. Any key absent from `craftcargo.toml` is filled in by craftcargo's
generator logic on the next `update` run.

---

## Dependency mapping

craftcargo ships a built-in table that maps well-known `*-sys` Cargo crates to their Ubuntu
`build-depends` equivalents:

| Cargo crate | Ubuntu package |
|-------------|---------------|
| `openssl-sys` | `libssl-dev` |
| `pcre2-sys` | `libpcre2-dev` |
| `sqlite3-sys` | `libsqlite3-dev` |
| `zlib-sys` | `zlib1g-dev` |
| `bzip2-sys` | `libbz2-dev` |
| `lzma-sys` | `liblzma-dev` |
| `libz-sys` | `zlib1g-dev` |
| `libgit2-sys` | `libgit2-dev` |
| `libssh2-sys` | `libssh2-1-dev` |
| `curl-sys` | `libcurl4-openssl-dev` |

You can extend or override this table in `craftcargo.toml`:

```toml
[dep-map]
"my-custom-sys" = "libmy-custom-dev (>= 2.0)"
```

Unmapped `*-sys` crates produce a `# FIXME: unmapped sys crate` comment in the generated
`debcraft.yaml` so they are easy to find and resolve manually.

---

## Vendoring dependencies

When `--vendor` is passed, craftcargo:

1. Runs `cargo vendor vendor/` to download all transitive dependencies.
2. Writes a `.cargo/config.toml` that points cargo at the `vendor/` directory.

The rust plugin picks up `.cargo/config.toml` automatically, so no changes to the
`debcraft.yaml` are needed beyond the vendored sources being present in the tree.

```sh
craftcargo generate zoxide 0.9.4 --vendor
ls vendor/    # all crate sources are here
```

The resulting `debcraft.yaml` parts section looks like:

```yaml
parts:
  zoxide:
    plugin: rust
    source: https://github.com/ajeetdsouza/zoxide
    source-type: git
    source-tag: $CRAFT_PROJECT_VERSION
    # vendor/ was populated by 'craftcargo generate --vendor'.
    # .cargo/config.toml points cargo at it; the rust plugin picks this up automatically.
    organize:
      (build)/target/release/zoxide: usr/bin/zoxide
```

This is the recommended approach for packages that need to build reproducibly in a
network-restricted environment such as Ubuntu's Launchpad build infrastructure.

---

## Multi-binary crates

When a `Cargo.toml` declares multiple `[[bin]]` targets, craftcargo emits one `packages`
entry per binary by default:

```yaml
packages:
  tool-foo: {}
  tool-bar: {}

parts:
  my-tool:
    plugin: rust
    source: https://github.com/example/my-tool
    source-type: git
    source-tag: $CRAFT_PROJECT_VERSION
    organize:
      (build)/target/release/foo: usr/bin/foo
      (build)/target/release/bar: usr/bin/bar
```

To collapse all binaries into a single package, add to `craftcargo.toml`:

```toml
[build]
merge-binaries = true
merged-package-name = "my-tool"
```

---

## Library crates

For crates that expose a Rust library (`[lib]` in `Cargo.toml`) craftcargo follows the
`librust-*-dev` Debian naming convention established by debcargo:

```yaml
name: rust-serde
version: 1.0.197-1
summary: a generic serialisation/deserialisation framework
description: |
  Serde is a framework for serialising and deserialising Rust data structures
  efficiently and generically.
base: ubuntu@24.04
maintainer: Alice Example <alice@example.com>
section: libdevel

packages:
  librust-serde-dev:                     # generated – librust-<crate>-dev convention
    multi-arch: foreign
    provides:
      - librust-serde-1-dev
      - librust-serde-1.0-dev
      - librust-serde-1.0.197-dev

parts:
  serde:
    plugin: rust
    source: https://github.com/serde-rs/serde
    source-type: git
    source-tag: v$CRAFT_PROJECT_VERSION
    organize:
      ".": usr/share/cargo/registry/serde-1.0.197
```

The `provides` list is generated automatically from the SemVer components so that cargo's
offline registry resolution works correctly inside a Debian build environment.

---

## Updating an existing descriptor

```sh
cd ~/packaging/ripgrep

# See what a version bump to 14.2.0 would change
craftcargo update 14.2.0 --diff

# Apply the update
craftcargo update 14.2.0

# craftcargo.toml overrides are preserved; only auto-generated fields change
```

After updating, review the diff and update `debian/changelog` as you normally would before
handing off to debcraft.

---

## Comparison with debcargo

| | debcargo | craftcargo |
|---|---|---|
| **Output** | `debian/` directory (control, rules, copyright, …) | Single `debcraft.yaml` |
| **Build tool** | dpkg-buildpackage / sbuild | debcraft |
| **Dependency vendoring** | External (`cargo vendor`) | Built-in (`--vendor` flag) |
| **Override mechanism** | `debian/` files edited by hand | `craftcargo.toml` |
| **Library packaging** | First-class (`librust-*-dev`) | Supported (`--lib` flag) |
| **Crates.io fetch** | Yes | Yes |
| **Local workspace** | No | Yes |
| **Update workflow** | `debcargo update` | `craftcargo update` |

craftcargo is not a replacement for debcargo. debcargo targets the full Debian/Ubuntu source
package format and is the right tool when you need a `.dsc` you can upload to Launchpad or
submit to the Debian archive. craftcargo targets the debcraft workflow and is optimised for
fast local iteration and CI pipelines that use debcraft as their build backend.

---

## Known limitations

- **Full copyright automation not yet implemented**: the `license:` key (an SPDX expression)
  is generated automatically from `Cargo.toml` and covers the package's overall license.
  However, it does not replace `debian/copyright`, which must map individual source paths
  to copyright holders and years. Maintain `debian/copyright` by hand as you would in a
  classic packaging workflow.
- **Lintian overrides not yet supported**: there is no `lintian:` section in
  `debcraft.yaml`. Add overrides to `debian/source/lintian-overrides` manually.
- **Shell completion auto-detection not yet implemented**: craftcargo detects completion
  scripts in the source tree and emits the appropriate `organize:` entries, but the
  debcraft build environment does not yet install them automatically. Review the generated
  `organize:` block and add missing entries by hand.
- **Workspace crates**: only the root crate of a Cargo workspace is packaged. Packaging
  individual workspace members is planned.
- **Procedural macros**: `proc-macro = true` crates are detected but the generated
  `debcraft.yaml` may require manual adjustment to handle the host-vs-target build split.
- **Custom build scripts**: `build.rs` scripts that link against libraries not in the
  `*-sys` mapping table will emit a `# FIXME: unmapped sys crate` comment and require a
  manual `build-packages` entry.
- **`[patch]` sections**: path and git patches in `Cargo.toml` are not carried over into
  the generated descriptor.

---

[debcraft]: https://documentation.ubuntu.com/snap-craft/en/latest/
[debcargo]: https://salsa.debian.org/rust-team/debcargo
