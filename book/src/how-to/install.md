# Install craftcargo

This guide covers the different ways to install craftcargo.

## From the Ubuntu archive

The recommended method for Ubuntu users:

```bash
sudo apt install craftcargo
```

## From source

Clone the repository and build:

```bash
git clone https://github.com/bepri/craftcargo
cd craftcargo
cargo build --release
sudo install -m 755 target/release/debcargo /usr/local/bin/craftcargo
```

### Build dependencies

Building from source requires:

- Rust toolchain (stable)
- `libssl-dev`
- `pkg-config`
- `libgit2-dev` (optional, for system libgit2)

On Ubuntu:

```bash
sudo apt install build-essential libssl-dev pkg-config
```
