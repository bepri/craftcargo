# Customise with overrides

craftcargo supports a `craftcargo.toml` override file that lets you customise the generated
`debcraft.yaml` without modifying it directly.

## Create an override file

Create a `craftcargo.toml` in the same directory as your output:

```toml
# Override the maintainer contact
maintainer = "Your Name <your.email@example.com>"

# Set a semver suffix for the package name
semver_suffix = true
```

## Use the override file

Pass it to the generate command:

```bash
craftcargo package-debcraft <crate-name> --config ./craftcargo.toml --directory ./output
```

## Common overrides

| Field | Purpose |
|-------|---------|
| `maintainer` | Set the Debian maintainer field |
| `uploaders` | List of additional uploaders |
| `semver_suffix` | Append major version to package name |
| `section` | Override the Debian section (default: `rust`) |
| `requires_root` | Set `Rules-Requires-Root` value |

## Preserve overrides on update

When you run `craftcargo update`, overrides from `craftcargo.toml` are preserved and
re-applied to the refreshed descriptor.

## Related

- [Update an existing descriptor](./update-descriptor.md)
- [Commands reference](../reference/commands.md)
