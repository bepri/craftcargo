# Demo

End-to-end demonstration of craftcargo generating `debcraft.yaml` from a Rust crate.

## Prerequisites

- `pv` installed (`sudo apt install pv`)
- craftcargo built (`cargo build --release` from repo root)

## Running

```bash
# Interactive mode (press ENTER to advance each step)
./run-demo.sh

# Auto-advance mode (no waiting)
./run-demo.sh -n
```

## What it shows

1. **Tool overview** — `craftcargo --help`
2. **Debian naming** — how crate names map to Debian source package names
3. **YAML generation** — `package-debcraft` against a real crate (tokio-util)
4. **Output inspection** — the generated `debcraft.yaml` structure
5. **Key sections** — metadata mapping and build parts
6. **Comparison** — single YAML file vs traditional ~15-file debcargo output

## Customising the demo

Edit `run-demo.sh` to:
- Change `TYPE_SPEED` for faster/slower typing simulation
- Swap the demo crate (e.g. use `serde`, `clap`, or `hyper` instead)
- Add/remove steps as needed

## Slide deck

See `slides-outline.md` for the accompanying presentation outline.
