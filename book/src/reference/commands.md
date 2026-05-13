# Commands

This page lists all craftcargo commands with their options.

## `package-debcraft`

Generate a `debcraft.yaml` from a crate.

```
craftcargo package-debcraft <crate-name> [options]
```

| Option | Description |
|--------|-------------|
| `--directory <path>` | Output directory for generated files |
| `--version <version>` | Specific crate version (defaults to latest) |
| `--config <path>` | Path to a `craftcargo.toml` override file |
| `--copyright-guess-harder` | Use heuristics for uncertain copyright info |

## `deb-src-name`

Print the Debian source package name for a crate.

```
craftcargo deb-src-name <crate-name> [version]
```

If `version` is given, the name includes a semver suffix.

## `extract`

Extract a crate without any packaging transformations.

```
craftcargo extract <crate-name> [options]
```

| Option | Description |
|--------|-------------|
| `--directory <path>` | Output directory |
| `--version <version>` | Specific crate version |

## `package`

Package a crate using the traditional debcargo method (generates `debian/` directory).

```
craftcargo package <crate-name> [options]
```

| Option | Description |
|--------|-------------|
| `--directory <path>` | Output directory |
| `--version <version>` | Specific crate version |
| `--config <path>` | Path to configuration file |
| `--changelog-ready` | Mark the changelog as release-ready |

## `build-order`

Print the transitive dependencies of a crate in topological order.

```
craftcargo build-order <crate-name>
```

## `deb-dependencies`

Print the dependencies of a crate in `debian/control` format.

```
craftcargo deb-dependencies <crate-name>
```

## `update`

Update the local crates.io registry index.

```
craftcargo update
```
