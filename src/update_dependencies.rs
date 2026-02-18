use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use cargo::core::Workspace;
use cargo::GlobalContext;

use debian_control::lossless::control::Control;
use debian_control::lossless::relations::Entry;

use crate::deb_dependencies::get_deb_dependencies;
use crate::debcargo_info;
use crate::debian::control::{base_deb_name, Package, DEV_SUFFIX};

#[derive(Debug, Clone, Parser)]
pub struct UpdateDependenciesArgs {
    /// Drop Rust development package dependencies that are not referenced by any workspace crate
    #[clap(long)]
    drop_unreferenced: bool,

    /// Include dependencies for crates that are present as workspace members
    #[clap(long)]
    include_local_crates: bool,

    /// Features to include in dependencies (can be specified multiple times)
    #[clap(long)]
    features: Vec<String>,

    /// Include all features in dependencies
    #[clap(long)]
    all_features: bool,

    /// Do not include default feature in dependencies
    #[clap(long)]
    no_default_features: bool,

    /// Allow prerelease versions of dependencies
    #[clap(long)]
    allow_prerelease_deps: bool,

    /// Include dev-dependencies
    #[clap(long)]
    include_dev_dependencies: bool,
}

/// Discover all Cargo.toml files for workspace crates.
///
/// Uses cargo's Workspace API to find all workspace member crates.
fn discover_workspace_crates() -> Result<(Vec<PathBuf>, HashSet<String>)> {
    let gctx = GlobalContext::default()?;
    let root_manifest = std::env::current_dir()?.join("Cargo.toml");
    let workspace = Workspace::new(&root_manifest, &gctx)?;

    let manifest_paths: Vec<PathBuf> = workspace
        .members()
        .map(|pkg| pkg.manifest_path().to_path_buf())
        .collect();

    let crate_names: HashSet<String> = workspace
        .members()
        .map(|pkg| pkg.name().to_string())
        .collect();

    Ok((manifest_paths, crate_names))
}

/// Check if a package name matches a local crate.
///
/// Uses a regex pattern to match package names like:
/// - librust-foo-dev (exact match)
/// - librust-foo-0.1-dev (versioned)
/// - librust-foo+feature-dev (feature variant)
/// - librust-foo-0.1+feature-dev (versioned with feature)
///
/// The regex pattern is:
/// librust-{crate_name}(?:-([0-9]+(?:\.[0-9]+(?:\.[0-9]+)?)?)?)?(\+[0-9a-zA-Z-]+)??-dev
fn is_local_package(pkg_name: &str, crate_name: &str) -> bool {
    // Build regex pattern for this crate name
    let pattern = format!(
        r"^librust-{crate_name}(?:-([0-9]+(?:\.[0-9]+(?:\.[0-9]+)?)?)?)?(\+[0-9a-zA-Z-]+)?-dev$"
    );

    // Safe to unwrap since we control the pattern
    let re = Regex::new(&pattern).unwrap();
    re.is_match(pkg_name)
}

/// Filter out dependencies that correspond to local workspace crates
fn filter_local_crate_dependencies(
    deps: &BTreeSet<String>,
    local_crate_names: &HashSet<String>,
) -> BTreeSet<String> {
    // Convert crate names to Debian format (underscores -> hyphens)
    let local_deb_names: HashSet<String> = local_crate_names
        .iter()
        .map(|name| base_deb_name(name))
        .collect();

    let mut filtered_deps = BTreeSet::new();

    for dep in deps {
        // Parse the dependency string using debian_control
        let entry: Entry = match dep.parse() {
            Ok(e) => e,
            Err(_) => {
                // If parsing fails, keep the dependency
                filtered_deps.insert(dep.clone());
                continue;
            }
        };

        // Check if any relation in this entry refers to a local crate
        let is_local = entry.relations().any(|rel| {
            let pkg_name = rel.name();
            local_deb_names
                .iter()
                .any(|crate_name| is_local_package(&pkg_name, crate_name))
        });

        if !is_local {
            filtered_deps.insert(dep.clone());
        } else {
            debcargo_info!("  Excluding local crate: {}", dep);
        }
    }

    filtered_deps
}

/// Update Build-Depends field in debian/control
fn update_build_dependencies(
    control_path: &str,
    new_deps: &BTreeSet<String>,
    drop_unreferenced: bool,
) -> Result<()> {
    let control = Control::from_file(control_path).context("Failed to read debian/control")?;

    let mut source = control.source().context("No source paragraph found")?;

    // Get or create Build-Depends
    let mut build_depends = source
        .build_depends()
        .unwrap_or_else(debian_control::lossless::relations::Relations::new);

    // Get package names from new_deps for quick lookup
    let new_pkg_names: HashSet<String> = new_deps
        .iter()
        .filter_map(|dep| dep.parse::<Entry>().ok())
        .flat_map(|entry| entry.relations().map(|rel| rel.name()).collect::<Vec<_>>())
        .collect();

    // Drop unreferenced librust-*-dev packages if requested
    let mut dropped_count = 0;
    if drop_unreferenced {
        let pkg_prefix = Package::pkg_prefix();
        let to_drop: Vec<String> = build_depends
            .entries()
            .flat_map(|entry| entry.relations().map(|rel| rel.name()).collect::<Vec<_>>())
            .filter(|pkg_name| {
                pkg_name.starts_with(pkg_prefix)
                    && pkg_name.ends_with(DEV_SUFFIX)
                    && !new_pkg_names.contains(pkg_name)
            })
            .collect();

        for pkg in &to_drop {
            if build_depends.drop_dependency(pkg) {
                debcargo_info!("  Dropping: {}", pkg);
                dropped_count += 1;
            }
        }
    }

    // Add/ensure new dependencies using ensure_relation
    let mut added_count = 0;
    for dep in new_deps {
        let entry: Entry = dep
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse dependency '{}': {}", dep, e))?;

        if build_depends.ensure_relation(entry) {
            debcargo_info!("  Adding: {}", dep);
            added_count += 1;
        }
    }

    // Update the Build-Depends field
    source.set_build_depends(&build_depends);

    // Write the updated control file
    std::fs::write(control_path, control.to_string()).context("Failed to write debian/control")?;

    if dropped_count > 0 {
        debcargo_info!(
            "\nDropped {} unreferenced {}*{} dependencies",
            dropped_count,
            Package::pkg_prefix(),
            DEV_SUFFIX
        );
    }
    debcargo_info!("Added {} new build dependencies", added_count);

    Ok(())
}

pub fn update_dependencies(args: UpdateDependenciesArgs) -> Result<()> {
    let control_path = "debian/control";

    // Discover all workspace crates
    debcargo_info!("Discovering workspace crates...");
    let (crate_manifests, crate_names) = discover_workspace_crates()?;
    debcargo_info!("Found {} workspace crates:", crate_manifests.len());
    for manifest in &crate_manifests {
        debcargo_info!("  - {}", manifest.display());
    }

    // Collect all dependencies from all crates
    let mut all_deps = BTreeSet::new();
    for manifest in &crate_manifests {
        debcargo_info!("\nAnalyzing dependencies for {}...", manifest.display());

        let (toolchain_deps, dependencies) = get_deb_dependencies(
            &manifest.clone(),
            &args.features.clone(),
            args.all_features,
            !args.no_default_features,
            args.allow_prerelease_deps,
            args.include_dev_dependencies,
        )?;

        let deps: BTreeSet<String> = toolchain_deps.into_iter().chain(dependencies).collect();
        debcargo_info!("  Found {} dependencies", deps.len());
        all_deps.extend(deps);
    }

    debcargo_info!(
        "\nTotal unique dependencies across all crates: {}",
        all_deps.len()
    );

    // Filter out local crate dependencies unless explicitly included
    let mut deps_to_add = all_deps;
    if !args.include_local_crates {
        debcargo_info!("\nFiltering out local workspace crates...");
        deps_to_add = filter_local_crate_dependencies(&deps_to_add, &crate_names);
        debcargo_info!(
            "Remaining dependencies after filtering: {}",
            deps_to_add.len()
        );
    }

    // Update debian/control with all dependencies
    debcargo_info!("\nUpdating {}...", control_path);
    update_build_dependencies(control_path, &deps_to_add, args.drop_unreferenced)?;

    debcargo_info!("\nDone!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local_package_exact_match() {
        assert!(is_local_package("librust-foo-dev", "foo"));
    }

    #[test]
    fn test_is_local_package_with_version() {
        assert!(is_local_package("librust-foo-0.1-dev", "foo"));
        assert!(is_local_package("librust-foo-0.1+default-dev", "foo"));
        assert!(is_local_package(
            "librust-desktop-edit-0.1+default-dev",
            "desktop-edit"
        ));
    }

    #[test]
    fn test_is_local_package_with_feature() {
        assert!(is_local_package("librust-foo+feature-dev", "foo"));
        assert!(is_local_package("librust-foo+default-dev", "foo"));
    }

    #[test]
    fn test_is_local_package_not_matching() {
        assert!(!is_local_package("librust-bar-dev", "foo"));
        assert!(!is_local_package("librust-foobar-dev", "foo"));
        // Should not match if it's not a -dev package
        assert!(!is_local_package("librust-foo", "foo"));
        // Known limitation: librust-foo-0-dev WILL match "foo" as a versioned package.
        // This is because we can't distinguish between a crate named "foo-0" and version
        // "0" of crate "foo". This is a fundamental flaw with Debian's package naming.
        assert!(is_local_package("librust-foo-0-dev", "foo"));
    }

    #[test]
    fn test_is_local_package_hyphenated_names() {
        assert!(is_local_package(
            "librust-systemd-unit-edit-0.1+default-dev",
            "systemd-unit-edit"
        ));
        assert!(is_local_package(
            "librust-desktop-edit+feature-dev",
            "desktop-edit"
        ));
    }

    #[test]
    fn test_filter_local_crate_dependencies_basic() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-foo-dev".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("foo".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
        assert!(!filtered.contains("librust-foo-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_with_version() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-foo-0.1+default-dev".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("foo".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
        assert!(!filtered.contains("librust-foo-0.1+default-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_hyphenated_crate_names() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-desktop-edit-0.1+default-dev".to_string());
        deps.insert("librust-systemd-unit-edit-0.1+default-dev".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("desktop-edit".to_string());
        local_crates.insert("systemd-unit-edit".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_with_features() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-foo+feature-dev".to_string());
        deps.insert("librust-foo+default-dev".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("foo".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_with_version_constraints() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-foo-0.1+default-dev (>= 0.1.3)".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("foo".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_underscore_conversion() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-my-crate-dev".to_string());

        let mut local_crates = HashSet::new();
        local_crates.insert("my_crate".to_string());

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("librust-serde-dev"));
    }

    #[test]
    fn test_filter_local_crate_dependencies_empty() {
        let deps = BTreeSet::new();
        let local_crates = HashSet::new();

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_local_crate_dependencies_no_local_crates() {
        let mut deps = BTreeSet::new();
        deps.insert("librust-serde-dev".to_string());
        deps.insert("librust-foo-dev".to_string());

        let local_crates = HashSet::new();

        let filtered = filter_local_crate_dependencies(&deps, &local_crates);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains("librust-serde-dev"));
        assert!(filtered.contains("librust-foo-dev"));
    }
}
