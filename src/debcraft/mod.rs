pub mod schema;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use chrono::Datelike;
use semver::Version;

use crate::config::{package_field_for_feature, Config, PackageKey};
use crate::crates::{all_dependencies_and_features, transitive_deps, CrateInfo};
use crate::debian::control::{deb_feature_name, deb_name, deb_upstream_version, dsc_name};
use crate::debian::{
    collapse_features, deb_deps, generate_homepage, normalize_feature_deps, reduce_provides,
    toolchain_deps, DebInfo,
};
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
    deb_info: &DebInfo,
    crate_info: &CrateInfo,
    config: &Config,
) -> Result<BTreeMap<String, DebcraftPackage>> {
    let mut packages = BTreeMap::new();

    let crate_name = crate_info.crate_name();
    let base_pkgname = deb_info.base_package_name();
    let name_suffix = deb_info.name_suffix();
    let pkgbase = match name_suffix {
        None => base_pkgname.to_string(),
        Some(suf) => format!("{base_pkgname}{suf}"),
    };

    let lib = crate_info.is_lib() && config.build_lib_package();
    let mut bins = crate_info.get_binary_targets();
    if lib && !bins.is_empty() && !config.build_bin_package() {
        bins.clear();
    }
    let bin_name = if config.bin_name == Config::default().bin_name {
        deb_info.base_package_name()
    } else {
        config.bin_name.as_str()
    };

    let (crate_summary, crate_description) = crate_info.get_summary_description();
    let summary_prefix = crate_summary.unwrap_or_else(|| format!("Rust crate \"{crate_name}\""));
    let description_prefix = {
        let tmp = crate_description.unwrap_or_default();
        if tmp.is_empty() {
            tmp
        } else {
            format!("{tmp}\n.\n")
        }
    };

    // 6a: library packages — one DebcraftPackage per feature (including base "").
    if lib {
        // 6a: feature/dep graph construction (reused unchanged from debian path).
        let features_with_deps = all_dependencies_and_features(crate_info.manifest());
        let working = normalize_feature_deps(features_with_deps)?;
        let (mut provides, reduced) = if config.collapse_features {
            collapse_features(&working)
        } else {
            reduce_provides(working)
        };

        // 6a: classify features into recommends (default-linked) vs. suggests.
        let mut rec_features: Vec<&str> = vec![];
        let mut sug_features: Vec<&str> = vec![];
        for (&feature, features) in &provides {
            if feature.is_empty() {
                continue;
            }
            if feature == "default" || features.contains(&"default") {
                rec_features.push(feature);
            } else {
                sug_features.push(feature);
            }
        }

        for (feature, (f_deps, o_deps)) in reduced {
            let pk = PackageKey::feature(feature);
            let f_provides = provides.remove(feature).unwrap();

            // 6a: per-feature summary and description (mirrors Package::new logic).
            let summary_suffix = if feature.is_empty() {
                " - Rust source code".to_string()
            } else {
                match f_provides.len() {
                    0 => format!(" - feature \"{feature}\""),
                    n => format!(" - feature \"{}\" and {} more", feature, n),
                }
            };
            let description_suffix = if feature.is_empty() {
                format!("Source code for Debianized Rust crate \"{crate_name}\"")
            } else {
                format!(
                    "This metapackage enables feature \"{}\" for the \
                     Rust {} crate, by pulling in any additional \
                     dependencies needed by that feature.{}",
                    feature,
                    crate_name,
                    match f_provides.len() {
                        0 => String::new(),
                        1 => format!(
                            "\n\nAdditionally, this package also provides the \
                             \"{}\" feature.",
                            f_provides[0],
                        ),
                        _ => format!(
                            "\n\nAdditionally, this package also provides the \
                             \"{}\", and \"{}\" features.",
                            f_provides[..f_provides.len() - 1].join("\", \""),
                            f_provides[f_provides.len() - 1],
                        ),
                    },
                )
            };

            // 6a: depends — explicit librust deps only, no substvars (${misc:Depends} etc. omitted).
            let mut depends = vec![];
            if !feature.is_empty() && !f_deps.contains(&"") {
                // Feature packages always need a direct dep on the base lib.
                depends.push(deb_name(&pkgbase));
            }
            depends.extend(f_deps.iter().map(|f| deb_feature_name(&pkgbase, f)));
            depends.extend(deb_deps(config.allow_prerelease_deps, &o_deps)?);

            // 6a: recommends/suggests on the base lib package only (feature pkgs get neither).
            let (recommends, suggests) = if feature.is_empty() {
                (
                    filter_provides_bare(&rec_features, &f_provides, &pkgbase),
                    filter_provides_bare(&sug_features, &f_provides, &pkgbase),
                )
            } else {
                (vec![], vec![])
            };

            // 6a: provides — bare names only; no "(= ${binary:Version})" clause.
            // debcraft injects the correct version at packaging time automatically.
            let feature_opt = if feature.is_empty() {
                None
            } else {
                Some(feature)
            };
            let pkg_provides = pkg_provides_bare(
                base_pkgname,
                name_suffix,
                crate_info.version(),
                feature_opt,
                &f_provides,
            );

            // 6a: breaks/replaces for semver-suffixed base lib packages.
            let mut breaks = vec![];
            let mut replaces = vec![];
            if name_suffix.is_some() && feature.is_empty() {
                let mut next = crate_info.version().clone();
                next.patch += 1;
                breaks.push(format!("{} (<< {}~)", deb_name(base_pkgname), next));
                replaces.push(format!("{} (<< {}~)", deb_name(base_pkgname), next));
            }
            if let Some(min_ver) = crate_info.rust_version().as_deref() {
                breaks.push(format!("rustc (<< {min_ver}~)"));
            }

            let pkg_name = if feature.is_empty() {
                deb_name(&pkgbase)
            } else {
                deb_feature_name(&pkgbase, feature)
            };

            // 6a: construct DebcraftPackage, then apply per-package config overrides.
            let mut pkg = DebcraftPackage {
                architectures: Some("any".to_string()),
                summary: Some(format!("{summary_prefix}{summary_suffix}")),
                description: Some(format!("{description_prefix}{description_suffix}")),
                depends,
                recommends,
                suggests,
                provides: pkg_provides,
                breaks,
                replaces,
                conflicts: vec![],
                section: None,
                multi_arch: Some("same".to_string()),
            };
            apply_package_overrides(
                &mut pkg,
                config,
                pk,
                &summary_suffix,
                &description_suffix,
                &f_provides,
            );
            packages.insert(pkg_name, pkg);
        }
        assert!(provides.is_empty());
    }

    // 6b: binary executable package.
    if !bins.is_empty() {
        let summary_suffix = String::new();
        let description_suffix = format!(
            "This package contains the following binaries built from the Rust crate\n\"{}\":\n - {}",
            crate_name,
            bins.join("\n - ")
        );

        // 6b: for semver-suffix packages, provide the unversioned binary name (bare, no version clause).
        let bin_provides = name_suffix
            .map(|_| vec![bin_name.to_string()])
            .unwrap_or_default();

        let bin_pkg_name = match name_suffix {
            None => bin_name.to_string(),
            Some(suf) => format!("{bin_name}{suf}"),
        };

        // 6b: depends is empty — substvars (${shlibs:Depends} etc.) are handled natively by debcraft.
        let mut pkg = DebcraftPackage {
            architectures: Some("any".to_string()),
            summary: Some(format!("{summary_prefix}{summary_suffix}")),
            description: Some(format!("{description_prefix}{description_suffix}")),
            depends: vec![],
            recommends: vec![],
            suggests: vec![],
            provides: bin_provides,
            breaks: vec![],
            replaces: vec![],
            conflicts: vec![],
            // 6b: section is a FIXME when a lib package also exists (mixed crate).
            section: if lib {
                Some("FIXME-(packages.\"(name)\".section)".to_string())
            } else {
                None
            },
            // 6b: multi_arch omitted — debcraft defaults to "no" for binary packages.
            multi_arch: None,
        };
        apply_package_overrides(
            &mut pkg,
            config,
            PackageKey::Bin,
            &summary_suffix,
            &description_suffix,
            &[],
        );
        packages.insert(bin_pkg_name, pkg);
    }

    // 6c: extra packages from [packages."extra+{name}"] in debcargo.toml.
    // Mirrors Package::new_extra() + apply_overrides(); all fields come from config.
    for configured in config.configured_packages() {
        if let PackageKey::Extra(package) = configured {
            let mut pkg = DebcraftPackage::default();
            apply_package_overrides(&mut pkg, config, configured, "", "", &[]);
            packages.insert(package.to_string(), pkg);
        }
    }

    Ok(packages)
}

/// Generate bare (no version clause) provides entries for a library package.
///
/// Implements the 6a provides list: bare `librust-foo-{version}-dev` names at
/// each version granularity (major, major.minor, major.minor.patch), plus one
/// entry per absorbed feature per suffix.  The package's own name is excluded.
///
/// Key difference from the Debian path (control.rs:388-403): debcraft injects
/// the package version at packaging time, so no `(= ${binary:Version})` clause.
fn pkg_provides_bare(
    basename: &str,
    name_suffix: Option<&str>,
    version: &Version,
    feature: Option<&str>,
    f_provides: &[&str],
) -> Vec<String> {
    let pkgbase = match name_suffix {
        None => basename.to_string(),
        Some(suf) => format!("{basename}{suf}"),
    };
    let version_suffixes = [
        String::new(),
        format!("-{}", version.major),
        format!("-{}.{}", version.major, version.minor),
        format!("-{}.{}.{}", version.major, version.minor, version.patch),
    ];
    let mut provides = vec![];
    for suffix in &version_suffixes {
        if name_suffix.is_some() && suffix.is_empty() {
            continue;
        }
        let p = format!("{basename}{suffix}");
        let entry = match feature.unwrap_or("") {
            "" => deb_name(&p),
            f => deb_feature_name(&p, f),
        };
        provides.push(entry);
        provides.extend(f_provides.iter().map(|f| deb_feature_name(&p, f)));
    }
    // The package does not provide itself.
    let self_name = match feature.unwrap_or("") {
        "" => deb_name(&pkgbase),
        f => deb_feature_name(&pkgbase, f),
    };
    provides.retain(|x| x != &self_name);
    provides
}

/// Filter feature names into bare deb package strings for recommends/suggests.
///
/// Used by 6a to build the base lib package's recommends and suggests lists.
fn filter_provides_bare(features: &[&str], f_provides: &[&str], pkgbase: &str) -> Vec<String> {
    features
        .iter()
        .filter(|f| !f_provides.contains(f))
        .map(|f| deb_feature_name(pkgbase, f))
        .collect()
}

/// Apply config overrides to a `DebcraftPackage`, mirroring `Package::apply_overrides()`.
///
/// Called at the end of 6a (lib features), 6b (binary), and 6c (extra packages).
fn apply_package_overrides(
    pkg: &mut DebcraftPackage,
    config: &Config,
    key: PackageKey<'_>,
    summary_suffix: &str,
    description_suffix: &str,
    f_provides: &[&str],
) {
    if let Some(section) = config.package_section(key) {
        pkg.section = Some(section.to_string());
    }
    // Per-package override replaces the whole string; global override replaces only the prefix.
    if let Some(per_pkg) = config.package_summary(key) {
        pkg.summary = Some(per_pkg.to_string());
    } else if let Some(global) = config.summary.as_deref() {
        pkg.summary = Some(format!("{global}{summary_suffix}"));
    }
    if let Some(per_pkg) = config.package_description(key) {
        pkg.description = Some(per_pkg.to_string());
    } else if let Some(global) = config.description.as_deref() {
        pkg.description = Some(format!("{global}{description_suffix}"));
    }
    pkg.depends.extend(package_field_for_feature(
        |x| config.package_depends(x),
        key,
        f_provides,
    ));
    pkg.recommends.extend(package_field_for_feature(
        |x| config.package_recommends(x),
        key,
        f_provides,
    ));
    pkg.suggests.extend(package_field_for_feature(
        |x| config.package_suggests(x),
        key,
        f_provides,
    ));
    pkg.provides.extend(package_field_for_feature(
        |x| config.package_provides(x),
        key,
        f_provides,
    ));
    pkg.breaks.extend(package_field_for_feature(
        |x| config.package_breaks(x),
        key,
        f_provides,
    ));
    pkg.replaces.extend(package_field_for_feature(
        |x| config.package_replaces(x),
        key,
        f_provides,
    ));
    pkg.conflicts.extend(package_field_for_feature(
        |x| config.package_conflicts(x),
        key,
        f_provides,
    ));
    if let Some(arch) = config.package_architecture(key) {
        pkg.architectures = Some(arch.join(" "));
    }
    if let Some(ma) = config.package_multi_arch(key) {
        pkg.multi_arch = Some(ma.to_string());
    }
}

fn build_debcraft_parts(
    crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    config: &Config,
    _packages: &BTreeMap<String, DebcraftPackage>,
) -> Result<BTreeMap<String, DebcraftPart>> {
    let mut parts = BTreeMap::new();

    let lib = crate_info.is_lib() && config.build_lib_package();
    let bins = crate_info.get_binary_targets();
    let has_bins = !bins.is_empty();

    if lib {
        // 7a: library crate — two parts: "crate" (dump) and "check" (cargo helper).

        // 7a / "crate" part: copies the source tree unconditionally using the dump plugin.
        // No build dependencies — the dump plugin needs none.
        let crate_part = DebcraftPart {
            plugin: "dump".to_string(),
            source: ".".to_string(),
            rust_channel: None,
            rust_features: vec![],
            build_packages: vec![],
            build_packages_arch: vec![],
            after: vec![],
        };
        parts.insert("crate".to_string(), crate_part);

        // 7a / "check" part: runs the crate tests via the debcraft cargo helper.
        // build_packages_arch carries toolchain deps + crate deps, nocheck-gated per 7c.
        let build_packages_arch = build_check_part_deps(crate_info, config)?;

        let check_part = DebcraftPart {
            plugin: "cargo".to_string(),
            source: ".".to_string(),
            rust_channel: None,
            rust_features: vec![],
            build_packages: vec![],
            build_packages_arch,
            after: vec!["crate".to_string()],
        };
        parts.insert("check".to_string(), check_part);
    } else if has_bins {
        // 7b: binary-only crate — single "rust" part using the craft-parts rust plugin.
        let build_packages_arch = build_bin_part_deps(crate_info, config)?;

        let rust_part = DebcraftPart {
            plugin: "rust".to_string(),
            source: ".".to_string(),
            // 7b: "none" tells the rust plugin to use the system toolchain.
            rust_channel: Some("none".to_string()),
            // 7b: features come from build deps; no explicit feature list needed.
            rust_features: vec![],
            // 7b: toolchain and crate deps go in build_packages for the rust plugin.
            build_packages: build_packages_arch
                .iter()
                .map(|e| match e {
                    schema::BuildPackageEntry::Simple(s) => s.clone(),
                    schema::BuildPackageEntry::WithProfile { package, .. } => package.clone(),
                })
                .collect(),
            build_packages_arch: vec![],
            after: vec![],
        };
        parts.insert("rust".to_string(), rust_part);
    }

    Ok(parts)
}

/// Build the `build_packages_arch` list for the library "check" part (7c).
///
/// Toolchain deps are always unconditional.  Crate deps are wrapped in a
/// `nocheck` profile unless `skip_nocheck` is set, which breaks bootstrapping
/// cycles — mirroring the `deb_dep_add_nocheck` logic in `prepare_debian_control()`.
fn build_check_part_deps(
    crate_info: &CrateInfo,
    config: &Config,
) -> Result<Vec<schema::BuildPackageEntry>> {
    let features_with_deps = all_dependencies_and_features(crate_info.manifest());
    let working = normalize_feature_deps(features_with_deps)?;
    let (default_features, default_deps) = transitive_deps(&working, "default")?;

    let extra_override_deps = package_field_for_feature(
        |x| config.package_depends(x),
        PackageKey::feature("default"),
        &default_features,
    );

    let skip_nocheck = config.skip_nocheck().unwrap_or(false);

    // 7c: toolchain deps are always Simple (never gated by nocheck).
    let mut entries: Vec<schema::BuildPackageEntry> =
        toolchain_deps(crate_info.rust_version().as_deref())
            .into_iter()
            .map(schema::BuildPackageEntry::Simple)
            .collect();

    // 7c: crate deps get a nocheck profile unless skip_nocheck is set.
    let crate_dep_strings: Vec<String> = deb_deps(config.allow_prerelease_deps, &default_deps)?
        .into_iter()
        .chain(extra_override_deps)
        .collect();

    for dep in crate_dep_strings {
        let entry = if skip_nocheck {
            // 7c: skip_nocheck — treat all deps as unconditional.
            schema::BuildPackageEntry::Simple(dep)
        } else {
            schema::BuildPackageEntry::WithProfile {
                package: dep,
                profiles: vec!["nocheck".to_string()],
            }
        };
        entries.push(entry);
    }

    // 7c: apply build_depends_excludes from SourceOverride.
    if let Some(excludes) = config.build_depends_excludes() {
        entries.retain(|e| {
            let pkg = match e {
                schema::BuildPackageEntry::Simple(s) => s.as_str(),
                schema::BuildPackageEntry::WithProfile { package, .. } => package.as_str(),
            };
            !excludes.iter().any(|ex| ex == pkg)
        });
    }

    // 7c: append any extra arch-specific build deps from SourceOverride.
    if let Some(extra) = config.build_depends_arch() {
        entries.extend(
            extra
                .iter()
                .map(|d| schema::BuildPackageEntry::Simple(d.clone())),
        );
    }

    Ok(entries)
}

/// Build the `build_packages` list for the binary-only "rust" part (7b).
///
/// For binary crates there is no bootstrapping concern, so all deps are simple
/// strings — no nocheck wrapping.
fn build_bin_part_deps(
    crate_info: &CrateInfo,
    config: &Config,
) -> Result<Vec<schema::BuildPackageEntry>> {
    let features_with_deps = all_dependencies_and_features(crate_info.manifest());
    let working = normalize_feature_deps(features_with_deps)?;
    let (default_features, default_deps) = transitive_deps(&working, "default")?;

    let extra_override_deps = package_field_for_feature(
        |x| config.package_depends(x),
        PackageKey::feature("default"),
        &default_features,
    );

    // 7b: toolchain deps + translated crate deps, all Simple (no nocheck needed).
    let entries: Vec<schema::BuildPackageEntry> =
        toolchain_deps(crate_info.rust_version().as_deref())
            .into_iter()
            .chain(deb_deps(config.allow_prerelease_deps, &default_deps)?)
            .chain(extra_override_deps)
            .map(schema::BuildPackageEntry::Simple)
            .collect();

    Ok(entries)
}

fn write_companion_files(
    crate_info: &CrateInfo,
    deb_info: &DebInfo,
    config_path: Option<&Path>,
    config: &Config,
    copyright_guess_harder: bool,
    out_dir: &Path,
) -> Result<()> {
    let maintainer = config.maintainer();
    let uploaders_owned = config.uploaders().cloned().unwrap_or_default();
    let uploaders: Vec<&str> = uploaders_owned.iter().map(String::as_str).collect();

    let year = chrono::Local::now().year();
    let year_range = (year, year);

    // 8a: write copyright file using the shared DEP-5 generator.
    {
        let dep5 = crate::debian::copyright::debian_copyright(
            out_dir,
            crate_info.manifest(),
            crate_info.manifest_path(),
            maintainer,
            &uploaders,
            year_range,
            copyright_guess_harder,
            config.excludes.as_deref().unwrap_or_default(),
        )?;
        let copyright_path = out_dir.join("copyright");
        if !copyright_path.exists() {
            fs::write(&copyright_path, format!("{dep5}"))?;
        }
    }

    // 8b: write cargo-checksum.json for the debcraft cargo helper.
    {
        let checksum = crate_info
            .checksum()
            .unwrap_or("Could not get crate checksum");
        let checksum_path = out_dir.join("cargo-checksum.json");
        if !checksum_path.exists() {
            fs::write(
                &checksum_path,
                format!(r#"{{"package":"{checksum}","files":{{}}}}"#),
            )?;
        }
    }

    // 8c: write one lintian-overrides file per non-empty feature package.
    {
        let features_with_deps = all_dependencies_and_features(crate_info.manifest());
        if let Ok(working) = normalize_feature_deps(features_with_deps) {
            let (_, reduced) = if config.collapse_features {
                collapse_features(&working)
            } else {
                reduce_provides(working)
            };
            let base_pkgname = deb_info.base_package_name();
            let name_suffix = deb_info.name_suffix();
            let pkgbase = match name_suffix {
                None => base_pkgname.to_string(),
                Some(suf) => format!("{base_pkgname}{suf}"),
            };
            for (feature, _) in reduced {
                if feature.is_empty() {
                    continue;
                }
                let pkg_name = deb_feature_name(&pkgbase, feature);
                let overrides_path = out_dir.join(format!("{pkg_name}.lintian-overrides"));
                if !overrides_path.exists() {
                    fs::write(
                        &overrides_path,
                        format!("{pkg_name} binary: empty-rust-library-declares-provides *"),
                    )?;
                }
            }
        }
    }

    // 8e: copy overlay files over generated ones (overlay wins).
    if let Some(overlay_dir) = config.overlay_debcraft_dir(config_path) {
        if overlay_dir.is_dir() {
            copy_overlay(&overlay_dir, out_dir)?;
        }
    }

    Ok(())
}

/// Recursively copy files from `src` into `dst`, overwriting any already-generated files.
fn copy_overlay(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_overlay(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
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
    fn spdx_single_license_unchanged() {
        assert_eq!(normalize_spdx_license("MIT"), "MIT");
        assert_eq!(normalize_spdx_license("Apache-2.0"), "Apache-2.0");
        assert_eq!(normalize_spdx_license("GPL-3.0-only"), "GPL-3.0-only");
    }

    #[test]
    fn spdx_and_expression_unchanged() {
        assert_eq!(
            normalize_spdx_license("MIT AND Apache-2.0"),
            "MIT AND Apache-2.0"
        );
    }

    #[test]
    fn spdx_with_exception_unchanged() {
        assert_eq!(
            normalize_spdx_license("Apache-2.0 WITH LLVM-exception"),
            "Apache-2.0 WITH LLVM-exception"
        );
    }

    #[test]
    fn spdx_complex_expression() {
        assert_eq!(
            normalize_spdx_license("MIT/Apache-2.0 AND GPL-2.0"),
            "MIT OR Apache-2.0 AND GPL-2.0"
        );
    }

    #[test]
    fn spdx_empty_string() {
        assert_eq!(normalize_spdx_license(""), "");
    }

    #[test]
    fn spdx_unlicense() {
        assert_eq!(
            normalize_spdx_license("MIT/Unlicense"),
            "MIT OR Unlicense"
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
    fn provides_bare_has_no_version_clause() {
        let version = semver::Version::new(1, 2, 3);
        let provides = pkg_provides_bare("foo", None, &version, None, &[]);
        assert!(!provides.is_empty());
        for entry in &provides {
            assert!(
                !entry.contains('$'),
                "provides entry contains substvar: {entry}"
            );
            assert!(
                !entry.contains('('),
                "provides entry has version clause: {entry}"
            );
        }
    }

    #[test]
    fn provides_bare_skips_unversioned_in_semver_suffix_pkg() {
        let version = semver::Version::new(1, 2, 3);
        // With name_suffix the unversioned "librust-foo-dev" must not appear.
        let provides = pkg_provides_bare("foo", Some("-1"), &version, None, &[]);
        assert!(
            !provides.iter().any(|p| p == "librust-foo-dev"),
            "unversioned entry must be absent in semver-suffix package; got: {provides:?}"
        );
        // But versioned entries should still be present.
        assert!(provides.iter().any(|p| p.contains("foo-1")));
    }

    #[test]
    fn provides_bare_does_not_contain_self() {
        let version = semver::Version::new(1, 2, 3);
        // The package's own name must not be in its provides list.
        let provides = pkg_provides_bare("foo", None, &version, None, &[]);
        assert!(!provides.contains(&"librust-foo-dev".to_string()));
    }

    #[test]
    fn issues_url_github_trailing_slash() {
        assert_eq!(
            derive_issues_url("https://github.com/foo/bar/"),
            Some("https://github.com/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_github_with_dot_git_and_trailing_slash() {
        assert_eq!(
            derive_issues_url("https://github.com/foo/bar.git/"),
            Some("https://github.com/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_gitlab_self_hosted() {
        assert_eq!(
            derive_issues_url("https://gitlab.freedesktop.org/mesa/mesa"),
            Some("https://gitlab.freedesktop.org/mesa/mesa/issues".to_string())
        );
    }

    #[test]
    fn issues_url_http_github() {
        assert_eq!(
            derive_issues_url("http://github.com/foo/bar"),
            Some("http://github.com/foo/bar/issues".to_string())
        );
    }

    #[test]
    fn issues_url_bitbucket_not_supported() {
        assert_eq!(
            derive_issues_url("https://bitbucket.org/user/repo"),
            None
        );
    }

    #[test]
    fn issues_url_codeberg_not_supported() {
        assert_eq!(
            derive_issues_url("https://codeberg.org/user/repo"),
            None
        );
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

    #[test]
    fn vcs_git_strips_trailing_slash() {
        assert_eq!(
            derive_vcs_git("https://github.com/foo/bar/"),
            Some("https://github.com/foo/bar".to_string())
        );
    }

    #[test]
    fn vcs_git_preserves_dot_git_suffix() {
        assert_eq!(
            derive_vcs_git("https://example.com/repo.git"),
            Some("https://example.com/repo.git".to_string())
        );
    }

    #[test]
    fn vcs_git_salsa_debian() {
        assert_eq!(
            derive_vcs_git("https://salsa.debian.org/rust-team/debcargo"),
            Some("https://salsa.debian.org/rust-team/debcargo".to_string())
        );
    }

    #[test]
    fn vcs_git_http_github() {
        assert_eq!(
            derive_vcs_git("http://github.com/foo/bar"),
            Some("http://github.com/foo/bar".to_string())
        );
    }

    #[test]
    fn vcs_git_gitlab_self_hosted() {
        assert_eq!(
            derive_vcs_git("https://gitlab.freedesktop.org/mesa/mesa"),
            Some("https://gitlab.freedesktop.org/mesa/mesa".to_string())
        );
    }

    #[test]
    fn vcs_git_none_for_plain_https() {
        assert_eq!(derive_vcs_git("https://crates.io/crates/serde"), None);
    }

    #[test]
    fn is_github_url_positive() {
        assert!(is_github_url("https://github.com/foo/bar"));
        assert!(is_github_url("http://github.com/foo/bar"));
    }

    #[test]
    fn is_github_url_negative() {
        assert!(!is_github_url("https://notgithub.com/foo/bar"));
        assert!(!is_github_url("https://gitlab.com/foo/bar"));
        assert!(!is_github_url("https://example.com/github.com/foo"));
        assert!(!is_github_url("ftp://github.com/foo/bar"));
    }

    #[test]
    fn is_gitlab_url_positive() {
        assert!(is_gitlab_url("https://gitlab.com/foo/bar"));
        assert!(is_gitlab_url("https://gitlab.example.org/foo/bar"));
        assert!(is_gitlab_url("https://gitlab.freedesktop.org/mesa/mesa"));
        assert!(is_gitlab_url("http://gitlab.com/foo/bar"));
    }

    #[test]
    fn is_gitlab_url_negative() {
        assert!(!is_gitlab_url("https://github.com/foo/bar"));
        assert!(!is_gitlab_url("https://notgitlab.com/foo/bar"));
        assert!(!is_gitlab_url("https://example.com/gitlab.com/foo"));
        assert!(!is_gitlab_url("ftp://gitlab.com/foo/bar"));
    }

    #[test]
    fn is_salsa_url_positive() {
        assert!(is_salsa_url("https://salsa.debian.org/rust-team/debcargo"));
        assert!(is_salsa_url("http://salsa.debian.org/foo/bar"));
    }

    #[test]
    fn is_salsa_url_negative() {
        assert!(!is_salsa_url("https://github.com/foo/bar"));
        assert!(!is_salsa_url("https://notsalsa.debian.org/foo/bar"));
        assert!(!is_salsa_url("https://salsa.example.com/foo/bar"));
        assert!(!is_salsa_url("ftp://salsa.debian.org/foo/bar"));
    }
}
