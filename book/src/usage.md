# Usage

## Generate a debcraft.yaml

```bash
# From a crates.io crate
craftcargo package-debcraft <crate-name> --directory <output-dir>

# Example
craftcargo package-debcraft tokio-util --directory ./tokio-util
```

## Other commands

```bash
# Get the Debian source package name for a crate
craftcargo deb-src-name <crate-name>

# Update the crates.io index
craftcargo update

# Extract a crate without transformations
craftcargo extract <crate-name> --directory <output-dir>

# Print transitive dependencies in topological order
craftcargo build-order <crate-name>

# Print dependencies in debian/control format
craftcargo deb-dependencies <crate-name>
```

## Options

Most commands accept:

- `--directory <path>` — output directory (defaults to current directory)
- `--version <version>` — specific crate version (defaults to latest)
- `--config <path>` — path to a `craftcargo.toml` override file

## The craftcargo.toml override file

You can create a `craftcargo.toml` alongside your output to customise the generated
`debcraft.yaml` without modifying it directly. Overrides are preserved when you run
`craftcargo update`.

See the [Introduction](./introduction.md#overrides-and-the-craftcargotoml-config-file) for full details on override fields.
