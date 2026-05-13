# How It Works

craftcargo bridges the Cargo ecosystem and Debian/Ubuntu packaging by automating the
generation of `debcraft.yaml` build descriptors.

## The Pipeline

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

## What It Does

1. **Fetches crate metadata** from the crates.io registry (or a local workspace)
2. **Resolves dependencies** and maps Cargo package names to Debian package names
3. **Normalises licences** from SPDX expressions to Debian-compatible format
4. **Derives URLs** — homepage, VCS, issues — from crate metadata
5. **Generates structured YAML** matching the debcraft schema exactly

## Key Design Principles

- **One-shot generation** — produces a `debcraft.yaml` that you version-control
- **Override-friendly** — `craftcargo.toml` lets you customise without modifying generated output
- **Idempotent updates** — `craftcargo update` refreshes the descriptor while preserving overrides
- **Built on debcargo** — reuses battle-tested registry logic and dependency resolution
