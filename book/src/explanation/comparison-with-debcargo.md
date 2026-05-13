# Comparison with debcargo

craftcargo builds on debcargo but produces fundamentally different output. This page
explains the relationship.

## What debcargo does

debcargo is the traditional tool for packaging Rust crates for Debian. It:

- Fetches crate source from crates.io
- Generates a full `debian/` directory (15+ files)
- Produces `debian/control`, `debian/rules`, `debian/copyright`, `debian/changelog`, etc.
- Outputs files directly consumable by `dpkg-buildpackage`

## What craftcargo does differently

craftcargo reuses debcargo's internal libraries for registry access and dependency
resolution, but instead of generating a `debian/` directory, it produces a single
`debcraft.yaml` file.

| Aspect | debcargo | craftcargo |
|--------|----------|------------|
| Output | `debian/` directory (15+ files) | Single `debcraft.yaml` |
| Build system | dpkg-buildpackage | debcraft |
| Files to maintain | Many | One |
| Override mechanism | Patch files, overrides in `debian/` | `craftcargo.toml` |
| Review surface | Large (multiple files) | Small (one YAML file) |

## Why the change

debcraft represents a newer approach to Debian/Ubuntu packaging that uses a declarative
YAML format instead of imperative scripts. Advantages include:

- **Simpler review** — one file to read instead of many
- **Version control friendly** — YAML diffs are clear and readable
- **Less boilerplate** — no `debian/rules` Makefile, no `debian/compat`
- **Reproducible** — declarative format leaves less room for side effects

## Shared internals

craftcargo uses these components from debcargo:

- Crates.io registry access and index management
- Crate metadata extraction
- Dependency resolution and Debian name mapping
- Version constraint translation
- Source tarball extraction

## When to use which

- Use **craftcargo** when targeting debcraft-based workflows (Ubuntu)
- Use **debcargo** when targeting traditional Debian packaging workflows
