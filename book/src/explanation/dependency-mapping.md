# Dependency mapping

This page explains how craftcargo translates Cargo dependencies into Debian package
dependencies.

## Naming convention

Cargo crate names map to Debian package names using a fixed convention:

| Crate type | Debian name pattern |
|------------|---------------------|
| Library | `librust-<name>-dev` |
| Library with feature | `librust-<name>-<feature>-dev` |
| Library with semver suffix | `librust-<name>-<major>-dev` |

Underscores in crate names become hyphens. Names are lowercased.

## Version translation

Cargo version requirements use semver syntax. These are translated to Debian versioned
dependencies:

| Cargo constraint | Debian equivalent |
|------------------|-------------------|
| `^1.2.3` | `>= 1.2.3, << 2.0.0~` |
| `~1.2.3` | `>= 1.2.3, << 1.3.0~` |
| `>=1.0, <2.0` | `>= 1.0, << 2.0~` |
| `=1.2.3` | `= 1.2.3` |
| `*` | (no version constraint) |

The `~` suffix on upper bounds ensures that pre-release versions of the boundary are
excluded, matching Debian's version comparison semantics.

## Feature dependencies

When a crate depends on another crate's feature, the dependency is expressed as a
dependency on the corresponding virtual package:

```
serde = { version = "1", features = ["derive"] }
```

becomes:

```
librust-serde-derive-dev (>= 1.0)
```

## Build vs runtime dependencies

- **Build-Depends** — all Cargo dependencies (Rust libraries are header-only)
- **Depends** — typically only `${misc:Depends}` and `${shlibs:Depends}` for binaries

## Toolchain dependencies

craftcargo adds a Rust toolchain dependency based on the crate's `rust-version` field
(MSRV) if present. Without an MSRV, it defaults to the current stable version.
