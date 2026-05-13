# Generate a debcraft.yaml

This guide shows how to generate a `debcraft.yaml` build descriptor from a Rust crate.

## From a crates.io crate

```bash
craftcargo package-debcraft <crate-name> --directory <output-dir>
```

For example:

```bash
craftcargo package-debcraft serde --directory ./serde-packaging
```

This fetches the latest version of `serde` from crates.io and generates the descriptor.

## Specify a version

To generate for a specific version:

```bash
craftcargo package-debcraft serde --version 1.0.193 --directory ./serde-packaging
```

## Specify a configuration file

If you have an existing `craftcargo.toml` with overrides:

```bash
craftcargo package-debcraft serde --config ./craftcargo.toml --directory ./serde-packaging
```

## Examine the output

The generated `debcraft.yaml` contains all fields that debcraft needs to build and package
the crate. Inspect it with:

```bash
cat ./serde-packaging/debcraft.yaml
```

## Related

- [Customise with overrides](./customise-with-overrides.md) for tweaking the generated output
- [debcraft.yaml schema reference](../reference/debcraft-yaml-schema.md) for field descriptions
