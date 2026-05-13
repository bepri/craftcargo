# How craftcargo works

craftcargo automates the conversion of Rust crate metadata into Debian packaging
descriptors. This page explains the overall architecture and processing pipeline.

## The pipeline

```
crates.io / local Cargo.toml
          │
          ▼
    craftcargo package-debcraft
          │
          ├─ 1. Fetch crate metadata
          ├─ 2. Resolve dependencies
          ├─ 3. Normalise licences
          ├─ 4. Derive URLs
          ├─ 5. Map to debcraft schema
          │
          ▼
     debcraft.yaml
          │
          ▼
   debcraft build → .deb package
```

## Stage 1: Fetch crate metadata

craftcargo uses the crates.io registry (or a local workspace) to obtain the crate's
`Cargo.toml` manifest. This provides the crate name, version, authors, licence, repository
URL, description, and dependency list.

## Stage 2: Resolve dependencies

Each Cargo dependency is mapped to its corresponding Debian package name using the
`librust-<name>-dev` convention. Version constraints from `Cargo.toml` are translated to
Debian versioned dependency syntax.

## Stage 3: Normalise licences

SPDX licence expressions from `Cargo.toml` are normalised for Debian compatibility:

- `MIT OR Apache-2.0` stays as-is (valid SPDX)
- `WITH` clauses (e.g. `Apache-2.0 WITH LLVM-exception`) are preserved
- Non-standard identifiers are flagged for manual review

## Stage 4: Derive URLs

craftcargo infers several URLs from the crate metadata:

- **Homepage** — prefers `homepage` field, falls back to crates.io URL
- **VCS** — derives Salsa/GitHub/GitLab URLs from the `repository` field
- **Issues** — appends `/issues` to the repository URL where appropriate

## Stage 5: Map to debcraft schema

The collected information is serialised into `debcraft.yaml` using serde, with field names
converted to kebab-case to match the debcraft specification.

## Design principles

- **One-shot generation** — craftcargo produces a complete descriptor in a single invocation
- **Deterministic output** — the same inputs always produce the same YAML
- **Override-friendly** — `craftcargo.toml` allows customisation without modifying the output
- **Built on debcargo** — reuses battle-tested registry and dependency logic
