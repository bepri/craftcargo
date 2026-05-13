# Installation

## From the Ubuntu archive (recommended)

```bash
sudo apt install craftcargo
```

## From source

```bash
git clone https://github.com/bepri/craftcargo
cd craftcargo
cargo build --release
sudo install -m 755 target/release/debcargo /usr/local/bin/craftcargo
```

## Prerequisites

- Rust toolchain (for building from source)
- `libssl-dev` and `pkg-config` (for the git2/openssl dependencies)
- An internet connection (to fetch crate metadata from crates.io)
