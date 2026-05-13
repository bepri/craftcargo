# Package your first crate

This tutorial walks you through generating a `debcraft.yaml` for a Rust crate, from
installation through to inspecting the output. By the end, you'll have a working build
descriptor ready for debcraft.

## Prerequisites

You need:

- A machine running Ubuntu (22.04 LTS or later)
- Internet access (to fetch crate metadata)

## Install craftcargo

Install from the Ubuntu archive:

```bash
sudo apt install craftcargo
```

Alternatively, build from source:

```bash
git clone https://github.com/bepri/craftcargo
cd craftcargo
cargo build --release
sudo install -m 755 target/release/debcargo /usr/local/bin/craftcargo
```

Verify the installation:

```bash
craftcargo --version
```

## Choose a crate

For this tutorial, we'll package `ripgrep` — a popular search tool written in Rust.

First, check what the Debian source package name would be:

```bash
craftcargo deb-src-name ripgrep
```

The output shows the conventional Debian source package name for this crate.

## Generate the debcraft.yaml

Run the `package-debcraft` command:

```bash
craftcargo package-debcraft ripgrep --directory ./ripgrep-packaging
```

craftcargo fetches the crate metadata, resolves dependencies, normalises the licence, and
writes a `debcraft.yaml` to the output directory.

## Inspect the output

List what was generated:

```bash
ls ./ripgrep-packaging/
```

View the debcraft.yaml:

```bash
cat ./ripgrep-packaging/debcraft.yaml
```

The YAML contains:

- **Package metadata** — name, version, summary, description
- **Source information** — homepage, VCS URLs, issues link
- **Build dependencies** — Rust toolchain and library deps
- **Parts** — build steps for debcraft

## What's next

You now have a working `debcraft.yaml`. From here you can:

- [Customise the output with overrides](../how-to/customise-with-overrides.md)
- [Build the package with debcraft](https://documentation.ubuntu.com/debcraft/)
- [Understand how the mapping works](../explanation/how-craftcargo-works.md)
