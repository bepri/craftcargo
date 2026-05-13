# Licence normalisation

craftcargo normalises SPDX licence expressions from Cargo crate metadata for use in Debian
packaging.

## Why normalisation is needed

Cargo crates declare licences using SPDX expressions in their `Cargo.toml`:

```toml
license = "MIT OR Apache-2.0"
```

Debian uses the [DEP-5 machine-readable copyright format](https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/),
which also uses SPDX identifiers but with specific formatting requirements.

## What craftcargo does

1. **Parses** the SPDX expression from the crate metadata
2. **Validates** each identifier against the SPDX licence list
3. **Preserves** compound expressions (`OR`, `AND`, `WITH`)
4. **Normalises** identifier casing to match SPDX canonical forms
5. **Flags** non-standard identifiers that need manual review

## Examples

| Cargo licence field | Normalised output |
|---------------------|-------------------|
| `MIT` | `MIT` |
| `MIT OR Apache-2.0` | `MIT OR Apache-2.0` |
| `Apache-2.0 WITH LLVM-exception` | `Apache-2.0 WITH LLVM-exception` |
| `GPL-3.0-only AND MIT` | `GPL-3.0-only AND MIT` |
| `MIT/Apache-2.0` | `MIT OR Apache-2.0` (legacy `/` syntax) |

## Edge cases

- **Empty licence field** — craftcargo produces an empty string and logs a warning
- **`license-file` only** — when the crate uses a file instead of an SPDX expression,
  craftcargo notes this for manual review
- **Non-standard identifiers** — passed through with a warning
