# Update an existing descriptor

When an upstream crate releases a new version, you can refresh the `debcraft.yaml` while
preserving your overrides.

## Update to the latest version

```bash
craftcargo package-debcraft <crate-name> --config ./craftcargo.toml --directory ./existing-output
```

This regenerates the `debcraft.yaml` using the latest crate version, applying any overrides
from your `craftcargo.toml`.

## Update to a specific version

```bash
craftcargo package-debcraft <crate-name> --version <new-version> --config ./craftcargo.toml --directory ./existing-output
```

## What gets preserved

- All fields defined in `craftcargo.toml` are re-applied
- The generated YAML is fully regenerated (not patched)

## What to check after updating

1. Review the diff of `debcraft.yaml` for unexpected changes
2. Check for new dependencies that may need packaging first
3. Verify licence information hasn't changed significantly
