# Package a multi-binary crate

Some Rust crates produce multiple binary executables. This guide covers how craftcargo
handles them.

## Generate the descriptor

```bash
craftcargo package-debcraft <crate-name> --directory ./output
```

craftcargo detects multiple `[[bin]]` targets in the crate's `Cargo.toml` and generates
appropriate packaging for each.

## How binaries are packaged

By default, all binaries from a crate are included in a single Debian package. The package
name is derived from the crate name following Debian conventions.

## Splitting binaries into separate packages

If you need individual binaries in separate packages, use override configuration in
`craftcargo.toml` to define per-package settings.

## Related

- [Customise with overrides](./customise-with-overrides.md)
- [debcraft.yaml schema](../reference/debcraft-yaml-schema.md)
