pub mod schema;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::crates::CrateInfo;
use crate::debian::control::{deb_upstream_version, dsc_name};
use crate::debian::{generate_homepage, DebInfo};
use crate::errors::Result;

use schema::{DebcraftPackage, DebcraftPart, DebcraftYaml};

/// Generate a `debcraft.yaml` file and companion files in `output_dir`.
///
/// This is the debcraft equivalent of `prepare_debian_folder()`.  Instead of
/// writing a full `debian/` tree it builds the schema in memory, serialises it
/// to YAML, and writes a small set of companion files alongside it.
#[allow(clippy::too_many_arguments)]
pub fn prepare_debcraft_yaml(
    crate_info: &mut CrateInfo,
    deb_info: &DebInfo,
    config_path: Option<&Path>,
    config: &Config,
    output_dir: &Path,
    copyright_guess_harder: bool,
) -> Result<()> {
    let mut yaml = build_debcraft_top_level(crate_info, deb_info, config)?;

    yaml.packages = build_debcraft_packages(deb_info, crate_info, config)?;

    yaml.parts = build_debcraft_parts(crate_info, deb_info, config, &yaml.packages)?;

    apply_source_overrides(&mut yaml, config);

    fs::create_dir_all(output_dir)?;
    let yaml_str = serde_yaml_ng::to_string(&yaml)?;
    fs::write(output_dir.join("debcraft.yaml"), yaml_str)?;

    write_companion_files(
        crate_info,
        deb_info,
        config_path,
        config,
        copyright_guess_harder,
        output_dir,
    )?;

    Ok(())
}

fn build_debcraft_top_level(
    crate_info: &CrateInfo,
    deb_info: &DebInfo,
    config: &Config,
) -> Result<DebcraftYaml> {
    let meta = crate_info.metadata();

    // 5a: source package name
    let name = dsc_name(deb_info.base_package_name());

    // 5b: initial Debian version
    let version = format!("{}-1", deb_info.deb_upstream_version());

    // 5c: summary / description (top-level gets the " - Rust source code" suffix)
    let (crate_summary, crate_description) = crate_info.get_summary_description();
    let summary = crate_summary.map(|s| format!("{s} - Rust source code"));
    let description = crate_description;

    // 5d / 5e: maintainer and uploaders
    let maintainer = config.maintainer().to_string();
    let uploaders = config.uploaders().map(|v| v.clone()).unwrap_or_default();

    // 5f: section
    let lib = crate_info.is_lib() && config.build_lib_package();
    let section = Some(if lib { "rust" } else { "FIXME" }.to_string());

    // 5g: contact — packager override, else the maintainer address
    let contact = config
        .contact
        .clone()
        .or_else(|| Some(config.maintainer().to_string()));

    // 5h: source-code URL
    let source_code = Some(generate_homepage(
        crate_info.crate_name(),
        &crate_info.version().to_string(),
        meta.homepage.as_deref(),
        meta.repository.as_deref(),
        config.crate_src_path.is_none(),
    ))
    .filter(|s| !s.is_empty());

    // 5i: vcs-git — config override first, then repository if it looks like a git URL
    let vcs_git = config
        .vcs_git()
        .map(str::to_string)
        .or_else(|| meta.repository.as_deref().and_then(derive_vcs_git));

    // 5j: license — normalise SPDX "/" separator to " OR "
    let license = meta.license.as_deref().map(normalize_spdx_license);

    // 5k: issues URL derived from repository
    let issues = meta.repository.as_deref().and_then(derive_issues_url);

    // 5l: custom source fields
    let plain_version = deb_upstream_version(crate_info.version(), None);
    let mut custom_source_fields = BTreeMap::new();
    custom_source_fields.insert(
        "X-Cargo-Crate".to_string(),
        crate_info.crate_name().to_string(),
    );
    custom_source_fields.insert("X-Cargo-Crate-Version".to_string(), plain_version);

    // 5m: rules-requires-root
    let rules_requires_root = config.requires_root.clone();

    // 5n: base image
    let base = config.base.clone();

    Ok(DebcraftYaml {
        name,
        version,
        summary,
        description,
        maintainer,
        uploaders,
        section,
        priority: None,
        contact,
        source_code,
        vcs_git,
        license,
        issues,
        base,
        custom_source_fields,
        rules_requires_root,
        parts: BTreeMap::new(),
        packages: BTreeMap::new(),
    })
}

/// Apply `[source]` config overrides on top of the computed top-level fields.
fn apply_source_overrides(yaml: &mut DebcraftYaml, config: &Config) {
    if let Some(section) = config.section() {
        yaml.section = Some(section.to_string());
    }
    // vcs_git override
    if let Some(vcs_git) = config.vcs_git() {
        yaml.vcs_git = Some(vcs_git.to_string());
    }
    // source-code can be overridden by vcs_browser or homepage in [source]
    if let Some(browser) = config.vcs_browser().or_else(|| config.homepage()) {
        yaml.source_code = Some(browser.to_string());
    }
}

/// Normalise an SPDX license expression from Cargo.toml.
///
/// Cargo uses "/" as an OR separator (pre-SPDX legacy); replace with " OR ".
fn normalize_spdx_license(license: &str) -> String {
    license.replace('/', " OR ")
}

/// If `repository` looks like a GitHub or GitLab URL, return its issues URL.
fn derive_issues_url(repository: &str) -> Option<String> {
    let url = repository.trim_end_matches('/').trim_end_matches(".git");
    // Don't double-append if already pointing at the issues page.
    if url.ends_with("/issues") {
        return Some(url.to_string());
    }
    if is_github_url(url) || is_gitlab_url(url) {
        return Some(format!("{url}/issues"));
    }
    None
}

/// If `repository` looks like a git URL, return it as a vcs-git value.
fn derive_vcs_git(repository: &str) -> Option<String> {
    let url = repository.trim_end_matches('/');
    if is_github_url(url) || is_gitlab_url(url) || is_salsa_url(url) || url.ends_with(".git") {
        return Some(url.to_string());
    }
    None
}

fn is_github_url(url: &str) -> bool {
    url.starts_with("https://github.com/") || url.starts_with("http://github.com/")
}

fn is_gitlab_url(url: &str) -> bool {
    // Match https://gitlab.<anything>/ — check the host, not a substring anywhere.
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .map_or(false, |rest| rest.starts_with("gitlab."))
}

fn is_salsa_url(url: &str) -> bool {
    url.starts_with("https://salsa.debian.org/") || url.starts_with("http://salsa.debian.org/")
}

fn build_debcraft_packages(
    _deb_info: &DebInfo,
    _crate_info: &CrateInfo,
    _config: &Config,
) -> Result<BTreeMap<String, DebcraftPackage>> {
    todo!("step 6: binary package generation")
}

fn build_debcraft_parts(
    _crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    _config: &Config,
    _packages: &BTreeMap<String, DebcraftPackage>,
) -> Result<BTreeMap<String, DebcraftPart>> {
    todo!("step 7: parts generation")
}

fn write_companion_files(
    _crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    _config_path: Option<&Path>,
    _config: &Config,
    _copyright_guess_harder: bool,
    _out_dir: &Path,
) -> Result<()> {
    todo!("step 8: companion files")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spdx_slash_normalized_to_or() {
        assert_eq!(
            normalize_spdx_license("MIT/Apache-2.0"),
            "MIT OR Apache-2.0"
        );
        assert_eq!(
            normalize_spdx_license("MIT/Apache-2.0/ISC"),
            "MIT OR Apache-2.0 OR ISC"
        );
    }

    #[test]
    fn spdx_already_valid_unchanged() {
        assert_eq!(
            normalize_spdx_license("MIT OR Apache-2.0"),
            "MIT OR Apache-2.0"
        );
    }

    #[test]
    fn issues_url_derived_from_github() {
        assert_eq!(
            derive_issues_url("https://github.com/foo/bar"),
            Some("https://github.com/foo/bar/issues".to_string())
        );
        // trailing slash and .git stripped before appending
        assert_eq!(
            derive_issues_url("https://github.com/foo/bar.git"),
            Some("https://github.com/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_derived_from_gitlab() {
        assert_eq!(
            derive_issues_url("https://gitlab.com/foo/bar"),
            Some("https://gitlab.com/foo/bar/issues".to_string())
        );
        assert_eq!(
            derive_issues_url("https://gitlab.example.org/foo/bar"),
            Some("https://gitlab.example.org/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_not_double_appended() {
        assert_eq!(
            derive_issues_url("https://github.com/foo/bar/issues"),
            Some("https://github.com/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_none_for_unknown_host() {
        assert_eq!(derive_issues_url("https://sr.ht/~user/repo"), None);
        assert_eq!(derive_issues_url("https://example.com/repo"), None);
        // substring-match trap: host is not github.com
        assert_eq!(derive_issues_url("https://notgithub.com/foo/bar"), None);
        assert_eq!(derive_issues_url("https://evil.com/github.com/foo"), None);
    }

    #[test]
    fn vcs_git_derived_for_known_hosts() {
        assert!(derive_vcs_git("https://github.com/foo/bar").is_some());
        assert!(derive_vcs_git("https://gitlab.com/foo/bar").is_some());
        assert!(derive_vcs_git("https://salsa.debian.org/rust-team/foo").is_some());
        assert!(derive_vcs_git("https://example.com/repo.git").is_some());
    }

    #[test]
    fn vcs_git_none_for_unknown_plain_url() {
        assert_eq!(derive_vcs_git("https://example.com/repo"), None);
        // substring-match trap: host contains "github.com" but isn't
        assert_eq!(derive_vcs_git("https://evil.com/github.com/foo"), None);
    }
}
