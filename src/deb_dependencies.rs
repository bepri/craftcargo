use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cargo::core::EitherManifest;
use cargo::core::SourceId;
use cargo::util::toml::read_manifest;
use cargo::GlobalContext;

use anyhow::Error;
use clap::Parser;

use crate::crates::all_dependencies_and_features_filtered;
use crate::crates::transitive_deps;
use crate::debian::deb_deps;
use crate::debian::toolchain_deps;

#[derive(Debug, Clone, Parser)]
pub struct DebDependenciesArgs {
    /// Cargo.toml for generating dependencies
    cargo_toml: PathBuf,
    /// Features to include in dependencies
    #[clap(long)]
    features: Vec<String>,
    /// Include all features in dependencies
    #[clap(long)]
    all_features: bool,
    /// Do not include default feature in dependencies
    #[clap(long="no-default-features", action=clap::ArgAction::SetFalse)]
    uses_default_features: bool,
    /// Allow prerelease versions of dependencies
    #[clap(long)]
    allow_prerelease_deps: bool,
    /// Include dev-dependencies
    #[clap(long)]
    include_dev_dependencies: bool,
}

/// Core function to get Debian dependencies for a Cargo.toml
///
/// This is the internal implementation that can be reused by multiple commands.
pub fn get_deb_dependencies(
    cargo_toml: &Path,
    features: &[String],
    all_features: bool,
    uses_default_features: bool,
    allow_prerelease_deps: bool,
    include_dev_dependencies: bool,
) -> Result<(Vec<String>, BTreeSet<String>), Error> {
    let cargo_toml = cargo_toml.canonicalize()?;
    let EitherManifest::Real(manifest) = read_manifest(
        &cargo_toml,
        SourceId::for_path(cargo_toml.parent().unwrap())?,
        &GlobalContext::default()?,
    )?
    else {
        debcargo_bail!("Manifest lacks project and package sections")
    };

    let deps_and_features =
        all_dependencies_and_features_filtered(&manifest, include_dev_dependencies);

    let feature_set = {
        let mut feature_set: std::collections::HashSet<_> = if all_features {
            deps_and_features.keys().copied().collect()
        } else {
            features
                .iter()
                .flat_map(|s| s.split_whitespace())
                .flat_map(|s| s.split(','))
                .filter(|s| !s.is_empty())
                .collect()
        };

        if uses_default_features {
            feature_set.insert("default");
        }

        feature_set.insert("");

        feature_set
    };
    let dependencies = {
        let mut dependencies = BTreeSet::<String>::new();
        for feature in &feature_set {
            if !deps_and_features.contains_key(feature) {
                debcargo_bail!("Unknown feature: {}", feature);
            }
            let (_, feature_deps) = transitive_deps(&deps_and_features, feature)?;
            dependencies.extend(deb_deps(allow_prerelease_deps, &feature_deps)?);
        }
        dependencies
    };
    let toolchain_deps = toolchain_deps(manifest.rust_version().map(ToString::to_string).as_deref());
    Ok((toolchain_deps, dependencies))
}

pub fn deb_dependencies(
    args: &DebDependenciesArgs,
) -> Result<(Vec<String>, BTreeSet<String>), Error> {
    get_deb_dependencies(
        &args.cargo_toml,
        &args.features,
        args.all_features,
        args.uses_default_features,
        args.allow_prerelease_deps,
        args.include_dev_dependencies,
    )
}
