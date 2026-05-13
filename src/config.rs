use indexmap::IndexMap;
use serde::de::IgnoredAny;
use serde::Deserialize;
use toml;

use crate::debian::control::PkgTestRestriction;
use crate::errors::Result;

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const RUST_MAINT: &str =
    "Debian Rust Maintainers <pkg-rust-maintainers@alioth-lists.debian.net>";

pub const DEFAULT_REPACK_SUFFIX: &str = "dfsg";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub bin: Option<bool>,
    pub bin_name: String,
    pub lib: Option<bool>,
    pub semver_suffix: bool,
    pub overlay: Option<PathBuf>,
    pub excludes: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub repack_suffix: Option<String>,
    pub allow_prerelease_deps: bool,
    pub crate_src_path: Option<PathBuf>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub maintainer: String,
    pub uploaders: Option<Vec<String>>,
    pub collapse_features: bool,
    pub requires_root: Option<String>,
    /// Packager-supplied contact address, distinct from maintainer when needed.
    pub contact: Option<String>,
    /// Base image for debcraft (e.g. `"ubuntu@24.04"`); omitted when not set.
    pub base: Option<String>,
    /// Separate base image used only during the build phase (9a).
    pub build_base: Option<String>,
    /// Override for the debcraft.yaml `issues` field; derived from repository URL when absent.
    pub issues: Option<String>,
    /// Override for the SPDX license string; defaults to `[package.license]` from Cargo.toml.
    pub license: Option<String>,
    /// Overlay directory for debcraft companion files; defaults to `debcraft/`
    /// relative to the config file if not set.
    pub overlay_debcraft: Option<PathBuf>,

    pub source: Option<SourceOverride>,
    pub packages: IndexMap<String, PackageOverride>,

    #[serde(flatten)]
    pub unknown_fields: HashMap<String, IgnoredAny>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SourceOverride {
    section: Option<String>,
    policy: Option<String>,
    homepage: Option<String>,
    vcs_git: Option<String>,
    vcs_browser: Option<String>,
    build_depends: Option<Vec<String>>,
    build_depends_arch: Option<Vec<String>>,
    build_depends_indep: Option<Vec<String>>,
    build_depends_excludes: Option<Vec<String>>,
    skip_nocheck: Option<bool>,

    #[serde(flatten)]
    pub unknown_fields: HashMap<String, IgnoredAny>,
}

impl SourceOverride {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        section: Option<String>,
        policy: Option<String>,
        homepage: Option<String>,
        vcs_git: Option<String>,
        vcs_browser: Option<String>,
        build_depends: Option<Vec<String>>,
        build_depends_arch: Option<Vec<String>>,
        build_depends_indep: Option<Vec<String>>,
        build_depends_excludes: Option<Vec<String>>,
        skip_nocheck: Option<bool>,
    ) -> Self {
        Self {
            section,
            policy,
            homepage,
            vcs_git,
            vcs_browser,
            build_depends,
            build_depends_arch,
            build_depends_indep,
            build_depends_excludes,
            skip_nocheck,
            unknown_fields: HashMap::new(),
        }
    }
}
#[derive(Deserialize, Debug, Clone, Default)]
pub struct PackageOverride {
    section: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    architecture: Option<Vec<String>>,
    multi_arch: Option<String>,
    depends: Option<Vec<String>>,
    recommends: Option<Vec<String>>,
    suggests: Option<Vec<String>>,
    provides: Option<Vec<String>>,
    breaks: Option<Vec<String>>,
    replaces: Option<Vec<String>>,
    conflicts: Option<Vec<String>>,
    extra_lines: Option<Vec<String>>,
    test_is_broken: Option<bool>,
    test_architecture: Option<Vec<String>>,
    test_depends: Option<Vec<String>>,
    test_restrictions: Option<Vec<PkgTestRestriction>>,

    #[serde(flatten)]
    pub unknown_fields: HashMap<String, IgnoredAny>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bin: None,
            bin_name: "<default>".to_string(),
            lib: None,
            semver_suffix: false,
            overlay: None,
            excludes: None,
            repack_suffix: None,
            whitelist: None,
            allow_prerelease_deps: false,
            crate_src_path: None,
            summary: None,
            description: None,
            maintainer: RUST_MAINT.to_string(),
            uploaders: None,
            collapse_features: false,
            source: None,
            packages: IndexMap::new(),
            requires_root: None,
            contact: None,
            base: None,
            build_base: None,
            issues: None,
            license: None,
            overlay_debcraft: None,
            unknown_fields: HashMap::new(),
        }
    }
}

impl Config {
    pub fn parse(src: &Path) -> Result<Config> {
        let mut config_file = File::open(src)?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;

        let config: Config = toml::from_str(&content)?;

        let mut unknown_fields = Vec::new();

        for field in config.unknown_fields.keys() {
            unknown_fields.push(field.clone());
        }

        if let Some(ref source) = config.source {
            for field in source.unknown_fields.keys() {
                unknown_fields.push(format!("source.{field}"));
            }
        }

        for field in config.packages.keys() {
            if PackageKey::from_key(field).is_none() {
                unknown_fields.push(format!("packages.{field}"));
            }
        }

        for (name, package) in &config.packages {
            for field in package.unknown_fields.keys() {
                unknown_fields.push(format!("packages.{name}.{field}"));
            }
        }

        if !unknown_fields.is_empty() {
            debcargo_warn!(
                "Warning: Unknown fields in {}: {:?}",
                src.display(),
                unknown_fields
            );
            debcargo_warn!("         These fields will be ignored. Please check for typos.");
        }

        Ok(config)
    }

    pub fn build_bin_package(&self) -> bool {
        self.bin.unwrap_or(!self.semver_suffix)
    }

    pub fn build_lib_package(&self) -> bool {
        self.lib.unwrap_or(true)
    }

    pub fn build_extra_package(&self) -> bool {
        self.configured_packages()
            .any(|p| matches!(p, PackageKey::Extra(_)))
    }

    pub fn overlay_dir(&self, config_path: Option<&Path>) -> Option<PathBuf> {
        Some(config_path?.parent()?.join(self.overlay.as_ref()?))
    }

    /// Overlay directory for debcraft companion files.
    ///
    /// Returns the explicit `overlay_debcraft` path if set; otherwise defaults
    /// to `debcraft/` relative to the config file (8e).
    pub fn overlay_debcraft_dir(&self, config_path: Option<&Path>) -> Option<PathBuf> {
        let base = config_path?.parent()?;
        if let Some(explicit) = &self.overlay_debcraft {
            Some(base.join(explicit))
        } else {
            Some(base.join("debcraft"))
        }
    }

    pub fn crate_src_path(&self, config_path: Option<&Path>) -> Option<PathBuf> {
        Some(config_path?.parent()?.join(self.crate_src_path.as_ref()?))
    }

    pub fn orig_tar_excludes(&self) -> Option<&Vec<String>> {
        self.excludes.as_ref()
    }

    pub fn orig_tar_whitelist(&self) -> Option<&Vec<String>> {
        self.whitelist.as_ref()
    }

    pub fn repack_suffix(&self) -> Option<&str> {
        self.repack_suffix
            .as_deref()
            .or_else(|| self.orig_tar_excludes().and(Some(DEFAULT_REPACK_SUFFIX)))
    }

    pub fn maintainer(&self) -> &str {
        self.maintainer.as_str()
    }

    pub fn uploaders(&self) -> Option<&Vec<String>> {
        self.uploaders.as_ref()
    }

    pub fn requires_root(&self) -> Option<&String> {
        self.requires_root.as_ref()
    }

    // Source shortcuts

    pub fn section(&self) -> Option<&str> {
        Some(self.source.as_ref()?.section.as_ref()?)
    }

    pub fn policy_version(&self) -> Option<&str> {
        Some(self.source.as_ref()?.policy.as_ref()?)
    }

    pub fn homepage(&self) -> Option<&str> {
        Some(self.source.as_ref()?.homepage.as_ref()?)
    }

    pub fn vcs_git(&self) -> Option<&str> {
        Some(self.source.as_ref()?.vcs_git.as_ref()?)
    }

    pub fn vcs_browser(&self) -> Option<&str> {
        Some(self.source.as_ref()?.vcs_browser.as_ref()?)
    }

    pub fn build_depends(&self) -> Option<&Vec<String>> {
        self.source.as_ref()?.build_depends.as_ref()
    }

    pub fn build_depends_arch(&self) -> Option<&Vec<String>> {
        self.source.as_ref()?.build_depends_arch.as_ref()
    }

    pub fn build_depends_indep(&self) -> Option<&Vec<String>> {
        self.source.as_ref()?.build_depends_indep.as_ref()
    }

    pub fn build_depends_excludes(&self) -> Option<&Vec<String>> {
        self.source.as_ref()?.build_depends_excludes.as_ref()
    }

    pub fn skip_nocheck(&self) -> Option<bool> {
        self.source.as_ref()?.skip_nocheck
    }

    // Packages accessors

    pub fn configured_packages(&'_ self) -> impl Iterator<Item = PackageKey<'_>> {
        self.packages.keys().filter_map(|k| PackageKey::from_key(k))
    }

    fn with_package<'a, T, F: FnOnce(&'a PackageOverride) -> Option<T>>(
        &'a self,
        key: PackageKey,
        f: F,
    ) -> Option<T> {
        self.packages.get(&key.key_string()[..]).and_then(f)
    }

    pub fn package_section(&self, key: PackageKey) -> Option<&str> {
        self.with_package(key, |pkg| pkg.section.as_deref())
    }

    pub fn package_summary(&self, key: PackageKey) -> Option<&str> {
        self.with_package(key, |pkg| pkg.summary.as_deref())
    }

    pub fn package_description(&self, key: PackageKey) -> Option<&str> {
        self.with_package(key, |pkg| pkg.description.as_deref())
    }

    pub fn package_architecture(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.architecture.as_ref())
    }

    pub fn package_multi_arch(&self, key: PackageKey) -> Option<&str> {
        self.with_package(key, |pkg| pkg.multi_arch.as_deref())
    }

    pub fn package_depends(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.depends.as_ref())
    }

    pub fn package_recommends(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.recommends.as_ref())
    }

    pub fn package_suggests(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.suggests.as_ref())
    }

    pub fn package_provides(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.provides.as_ref())
    }

    pub fn package_breaks(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.breaks.as_ref())
    }

    pub fn package_replaces(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.replaces.as_ref())
    }

    pub fn package_conflicts(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.conflicts.as_ref())
    }

    pub fn package_extra_lines(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.extra_lines.as_ref())
    }

    pub fn package_test_is_broken(&self, key: PackageKey) -> Option<bool> {
        self.with_package(key, |pkg| pkg.test_is_broken)
    }

    pub fn package_test_restrictions(&self, key: PackageKey) -> Option<&Vec<PkgTestRestriction>> {
        self.with_package(key, |pkg| pkg.test_restrictions.as_ref())
    }

    pub fn package_test_architecture(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.test_architecture.as_ref())
    }

    pub fn package_test_depends(&self, key: PackageKey) -> Option<&Vec<String>> {
        self.with_package(key, |pkg| pkg.test_depends.as_ref())
    }
}

pub fn package_field_for_feature<'a, 'b, F: Fn(PackageKey) -> Option<&'a Vec<String>>>(
    get_field: F,
    feature: PackageKey<'b>,
    f_provides: &'b [&'b str],
) -> impl Iterator<Item = String> + use<'a, 'b, F> {
    Some(feature)
        .into_iter()
        .chain(f_provides.iter().map(|s| PackageKey::feature(s)))
        .flat_map(move |f| get_field(f).into_iter().flatten())
        .map(ToString::to_string)
}

#[derive(Clone, Copy)]
pub enum PackageKey<'a> {
    Bin,
    BareLib,
    FeatureLib(&'a str),
    Extra(&'a str),
}

impl<'a> PackageKey<'a> {
    pub fn feature(f: &'a str) -> PackageKey<'a> {
        use self::PackageKey::{BareLib, FeatureLib};
        if f.is_empty() {
            BareLib
        } else {
            FeatureLib(f)
        }
    }

    pub fn from_key(k: &'a str) -> Option<PackageKey<'a>> {
        use self::PackageKey::{BareLib, Bin, Extra, FeatureLib};
        Some(match k {
            "bin" => Bin,
            "lib" => BareLib,
            _ => {
                if let Some(feature) = k.strip_prefix("lib+") {
                    FeatureLib(feature)
                } else if let Some(package) = k.strip_prefix("extra+") {
                    Extra(package)
                } else {
                    return None;
                }
            }
        })
    }

    fn key_string(&self) -> Cow<'static, str> {
        use self::PackageKey::{BareLib, Bin, Extra, FeatureLib};
        match self {
            Bin => "bin".into(),
            BareLib => "lib".into(),
            FeatureLib(feature) => format!("lib+{feature}").into(),
            Extra(package) => format!("extra+{package}").into(),
        }
    }
}

pub fn testing_ignore_debpolv() -> bool {
    std::env::var_os("DEBCARGO_TESTING_IGNORE_DEBIAN_POLICY_VIOLATION").as_deref()
        == Some(OsStr::new("1"))
}

pub fn testing_ruzt() -> bool {
    std::env::var_os("DEBCARGO_TESTING_RUZT").as_deref() == Some(OsStr::new("1"))
}
