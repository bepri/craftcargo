# Package a library crate

Library crates produce `-dev` packages in Debian. This guide covers the specifics.

## Generate the descriptor

```bash
craftcargo package-debcraft <library-crate> --directory ./output
```

craftcargo automatically detects that the crate is a library and adjusts the output
accordingly.

## What's different for libraries

- The binary package name uses the `librust-<name>-dev` convention
- Features generate additional virtual packages
- The package provides `librust-<name>-<feature>-dev` for each feature
- Multi-arch is set to `same`

## Handling features

Each Cargo feature maps to a virtual Debian package. Dependencies on features are expressed
as dependencies on these virtual packages.

For a crate with features `derive` and `alloc`:

```
librust-serde-dev
librust-serde-derive-dev
librust-serde-alloc-dev
```

## Related

- [Dependency mapping](../explanation/dependency-mapping.md) for how deps are resolved
- [Cargo → debcraft field mapping](../reference/cargo-debcraft-mapping.md)
