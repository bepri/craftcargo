# craftcargo

**craftcargo** is a command-line tool that automates the creation of
[`debcraft.yaml`](https://documentation.ubuntu.com/debcraft/) build descriptors for Rust
programs.

It reads a crate's `Cargo.toml` — either from a local workspace or directly from
[crates.io](https://crates.io) — and produces a ready-to-use `debcraft.yaml` that encodes
all the information debcraft needs to build and package the program for Ubuntu.

craftcargo does for debcraft what [debcargo](https://salsa.debian.org/rust-team/debcargo)
does for classic Debian source packages: it bridges the gap between the Cargo ecosystem and
the Debian packaging workflow.

---

## In this documentation

|  |  |
|--|--|
| **[Tutorial](./tutorial/package-first-crate.md)** | A hands-on lesson that walks you through packaging a Rust crate from start to finish |
| **[How-to guides](./how-to/install.md)** | Step-by-step directions for specific tasks — installation, generation, overrides |
| **[Reference](./reference/commands.md)** | Technical descriptions of commands, schema fields, and mappings |
| **[Explanation](./explanation/how-craftcargo-works.md)** | Discussion and context on how craftcargo works and why |

---

## Project and community

craftcargo is an open source project that welcomes contributions, suggestions, and
constructive feedback.

- [Source code on GitHub](https://github.com/bepri/craftcargo)
- [File a bug or feature request](https://github.com/bepri/craftcargo/issues)
- [Hacking guide](./development/hacking.md)
