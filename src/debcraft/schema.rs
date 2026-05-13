//! Serde-serialisable structs mirroring the debcraft.yaml schema.
//!
//! All fields use Rust snake_case and are renamed to kebab-case in YAML output.

use std::collections::BTreeMap;

use serde::Serialize;

/// Root structure of a debcraft.yaml file.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DebcraftYaml {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub maintainer: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uploaders: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Only serialised when non-default (debcraft default is "optional").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    /// Maps to the Vcs-Browser equivalent in debcraft (source code URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcs_git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Custom source stanza fields, e.g. X-Cargo-Crate and X-Cargo-Crate-Version.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub custom_source_fields: BTreeMap<String, String>,
    /// Rules-Requires-Root equivalent; omit when debcraft default ("no") applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_requires_root: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parts: BTreeMap<String, DebcraftPart>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub packages: BTreeMap<String, DebcraftPackage>,
}

/// A single build part in debcraft.yaml.
///
/// Library crates produce two parts: a `crate` part (dump plugin) and a
/// `check` part (carrying nocheck-gated dependencies). Binary crates produce
/// a single part using the `rust` plugin.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DebcraftPart {
    pub plugin: String,
    pub source: String,
    /// `"none"` uses the system toolchain; omit to let debcraft choose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_channel: Option<String>,
    /// Cargo features to enable when building a binary crate.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rust_features: Vec<String>,
    /// Build-time dependencies that are always required.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_packages: Vec<String>,
    /// Architecture-specific build dependencies, potentially with profile annotations.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_packages_arch: Vec<BuildPackageEntry>,
    /// Parts that this part must wait for (craft-parts `after` equivalent).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
}

/// A build-package entry, optionally restricted to specific build profiles.
///
/// `Simple` serialises as a plain string; `WithProfile` serialises as an
/// object with a `profiles` list, representing the `<!nocheck>` equivalent.
#[derive(Serialize)]
#[serde(untagged)]
pub enum BuildPackageEntry {
    Simple(String),
    WithProfile {
        package: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        profiles: Vec<String>,
    },
}

/// A binary package entry in debcraft.yaml.
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DebcraftPackage {
    /// Architecture constraint; `"any"` for all Rust packages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architectures: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommends: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggests: Vec<String>,
    /// Bare package names only — debcraft injects the version automatically.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provides: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub breaks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub replaces: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Always `"same"` for library packages; omit for others (debcraft default is "no").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_arch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_yaml<T: Serialize>(v: &T) -> serde_yaml_ng::Value {
        serde_yaml_ng::to_value(v).expect("serialisation failed")
    }

    // BuildPackageEntry: the untagged enum is the trickiest part of the schema.

    #[test]
    fn build_package_entry_simple_is_plain_string() {
        let entry = BuildPackageEntry::Simple("librust-foo-dev (>= 1.0)".to_string());
        let v = to_yaml(&entry);
        assert_eq!(
            v,
            serde_yaml_ng::Value::String("librust-foo-dev (>= 1.0)".to_string())
        );
    }

    #[test]
    fn build_package_entry_with_profile_is_object() {
        let entry = BuildPackageEntry::WithProfile {
            package: "librust-foo-dev".to_string(),
            profiles: vec!["nocheck".to_string()],
        };
        let v = to_yaml(&entry);
        assert_eq!(
            v["package"],
            serde_yaml_ng::Value::String("librust-foo-dev".to_string())
        );
        assert_eq!(
            v["profiles"][0],
            serde_yaml_ng::Value::String("nocheck".to_string())
        );
    }

    #[test]
    fn build_package_entry_with_empty_profiles_omits_profiles_key() {
        let entry = BuildPackageEntry::WithProfile {
            package: "librust-foo-dev".to_string(),
            profiles: vec![],
        };
        let v = to_yaml(&entry);
        assert!(
            v.get("profiles").is_none(),
            "empty profiles should be omitted"
        );
    }

    #[test]
    fn build_package_entry_with_multiple_profiles() {
        let entry = BuildPackageEntry::WithProfile {
            package: "librust-foo-dev".to_string(),
            profiles: vec!["nocheck".to_string(), "cross".to_string()],
        };
        let v = to_yaml(&entry);
        assert_eq!(
            v["profiles"][0],
            serde_yaml_ng::Value::String("nocheck".to_string())
        );
        assert_eq!(
            v["profiles"][1],
            serde_yaml_ng::Value::String("cross".to_string())
        );
    }

    // Kebab-case renaming: spot-check the fields most likely to be misnamed.

    #[test]
    fn debcraft_part_uses_kebab_case_keys() {
        let part = DebcraftPart {
            plugin: "cargo".to_string(),
            source: ".".to_string(),
            rust_channel: Some("none".to_string()),
            rust_features: vec!["default".to_string()],
            build_packages: vec!["libssl-dev".to_string()],
            build_packages_arch: vec![],
            after: vec![],
        };
        let v = to_yaml(&part);
        assert!(
            v.get("rust-channel").is_some(),
            "rust_channel should serialise as rust-channel"
        );
        assert!(
            v.get("rust-features").is_some(),
            "rust_features should serialise as rust-features"
        );
        assert!(
            v.get("build-packages").is_some(),
            "build_packages should serialise as build-packages"
        );
        assert!(
            v.get("rustChannel").is_none(),
            "camelCase key must not appear"
        );
    }

    #[test]
    fn debcraft_part_omits_empty_vecs() {
        let part = DebcraftPart {
            plugin: "rust".to_string(),
            source: "https://example.com".to_string(),
            rust_channel: None,
            rust_features: vec![],
            build_packages: vec![],
            build_packages_arch: vec![],
            after: vec![],
        };
        let v = to_yaml(&part);
        assert!(
            v.get("rust-channel").is_none(),
            "None rust_channel should be omitted"
        );
        assert!(
            v.get("rust-features").is_none(),
            "empty rust_features should be omitted"
        );
        assert!(
            v.get("build-packages").is_none(),
            "empty build_packages should be omitted"
        );
        assert!(
            v.get("build-packages-arch").is_none(),
            "empty build_packages_arch should be omitted"
        );
        assert!(
            v.get("after").is_none(),
            "empty after should be omitted"
        );
    }

    #[test]
    fn debcraft_part_includes_non_empty_after() {
        let part = DebcraftPart {
            plugin: "rust".to_string(),
            source: ".".to_string(),
            rust_channel: None,
            rust_features: vec![],
            build_packages: vec![],
            build_packages_arch: vec![],
            after: vec!["crate".to_string()],
        };
        let v = to_yaml(&part);
        assert!(v.get("after").is_some(), "non-empty after should be present");
        assert_eq!(
            v["after"][0],
            serde_yaml_ng::Value::String("crate".to_string())
        );
    }

    #[test]
    fn debcraft_yaml_kebab_case_top_level_fields() {
        let yaml = DebcraftYaml {
            name: "rust-foo".to_string(),
            version: "1.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "Test <test@example.com>".to_string(),
            uploaders: vec![],
            section: None,
            priority: None,
            contact: None,
            source_code: Some("https://example.com".to_string()),
            vcs_git: Some("https://salsa.debian.org/rust-team/foo".to_string()),
            license: None,
            issues: None,
            base: None,
            custom_source_fields: {
                let mut m = BTreeMap::new();
                m.insert("X-Cargo-Crate".to_string(), "foo".to_string());
                m
            },
            rules_requires_root: Some("no".to_string()),
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert!(v.get("source-code").is_some(), "source_code -> source-code");
        assert!(v.get("vcs-git").is_some(), "vcs_git -> vcs-git");
        assert!(
            v.get("custom-source-fields").is_some(),
            "custom_source_fields -> custom-source-fields"
        );
        assert!(
            v.get("rules-requires-root").is_some(),
            "rules_requires_root -> rules-requires-root"
        );
    }

    #[test]
    fn debcraft_yaml_required_fields_always_present() {
        let yaml = DebcraftYaml {
            name: "rust-bar".to_string(),
            version: "2.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "Maintainer <m@example.com>".to_string(),
            uploaders: vec![],
            section: None,
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: None,
            issues: None,
            base: None,
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert_eq!(
            v["name"],
            serde_yaml_ng::Value::String("rust-bar".to_string())
        );
        assert_eq!(
            v["version"],
            serde_yaml_ng::Value::String("2.0.0-1".to_string())
        );
        assert_eq!(
            v["maintainer"],
            serde_yaml_ng::Value::String("Maintainer <m@example.com>".to_string())
        );
    }

    #[test]
    fn debcraft_yaml_summary_and_description_when_present() {
        let yaml = DebcraftYaml {
            name: "rust-baz".to_string(),
            version: "0.1.0-1".to_string(),
            summary: Some("A short summary".to_string()),
            description: Some("A longer description\nwith multiple lines.".to_string()),
            maintainer: "Test <t@t.com>".to_string(),
            uploaders: vec![],
            section: Some("rust".to_string()),
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: Some("MIT".to_string()),
            issues: None,
            base: Some("ubuntu@24.04".to_string()),
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert_eq!(
            v["summary"],
            serde_yaml_ng::Value::String("A short summary".to_string())
        );
        assert!(v.get("description").is_some());
        assert_eq!(
            v["section"],
            serde_yaml_ng::Value::String("rust".to_string())
        );
        assert_eq!(
            v["license"],
            serde_yaml_ng::Value::String("MIT".to_string())
        );
        assert_eq!(
            v["base"],
            serde_yaml_ng::Value::String("ubuntu@24.04".to_string())
        );
    }

    #[test]
    fn debcraft_yaml_uploaders_omitted_when_empty() {
        let yaml = DebcraftYaml {
            name: "rust-x".to_string(),
            version: "1.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "M <m@m.com>".to_string(),
            uploaders: vec![],
            section: None,
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: None,
            issues: None,
            base: None,
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert!(v.get("uploaders").is_none(), "empty uploaders should be omitted");
    }

    #[test]
    fn debcraft_yaml_uploaders_present_when_nonempty() {
        let yaml = DebcraftYaml {
            name: "rust-x".to_string(),
            version: "1.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "M <m@m.com>".to_string(),
            uploaders: vec!["Alice <a@a.com>".to_string()],
            section: None,
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: None,
            issues: None,
            base: None,
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert!(v.get("uploaders").is_some());
        assert_eq!(
            v["uploaders"][0],
            serde_yaml_ng::Value::String("Alice <a@a.com>".to_string())
        );
    }

    // skip_serializing_if: verify that empty BTreeMap fields are omitted.

    #[test]
    fn empty_custom_source_fields_omitted() {
        let yaml = DebcraftYaml {
            name: "rust-foo".to_string(),
            version: "1.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "Test <test@example.com>".to_string(),
            uploaders: vec![],
            section: None,
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: None,
            issues: None,
            base: None,
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let v = to_yaml(&yaml);
        assert!(
            v.get("custom-source-fields").is_none(),
            "empty map should be omitted"
        );
        assert!(
            v.get("packages").is_none(),
            "empty packages map should be omitted"
        );
    }

    // DebcraftPackage serialisation tests

    #[test]
    fn debcraft_package_default_is_empty() {
        let pkg = DebcraftPackage::default();
        let v = to_yaml(&pkg);
        // Default package should have all fields omitted
        assert!(v.get("architectures").is_none());
        assert!(v.get("summary").is_none());
        assert!(v.get("description").is_none());
        assert!(v.get("depends").is_none());
        assert!(v.get("recommends").is_none());
        assert!(v.get("suggests").is_none());
        assert!(v.get("provides").is_none());
        assert!(v.get("breaks").is_none());
        assert!(v.get("replaces").is_none());
        assert!(v.get("conflicts").is_none());
        assert!(v.get("section").is_none());
        assert!(v.get("multi-arch").is_none());
    }

    #[test]
    fn debcraft_package_with_all_fields() {
        let pkg = DebcraftPackage {
            architectures: Some("any".to_string()),
            summary: Some("A package summary".to_string()),
            description: Some("Detailed description".to_string()),
            depends: vec!["libc6".to_string()],
            recommends: vec!["bash-completion".to_string()],
            suggests: vec!["docs".to_string()],
            provides: vec!["virtual-pkg".to_string()],
            breaks: vec!["old-pkg (<< 2.0)".to_string()],
            replaces: vec!["old-pkg (<< 2.0)".to_string()],
            conflicts: vec!["conflicting-pkg".to_string()],
            section: Some("utils".to_string()),
            multi_arch: Some("same".to_string()),
        };
        let v = to_yaml(&pkg);
        assert_eq!(
            v["architectures"],
            serde_yaml_ng::Value::String("any".to_string())
        );
        assert_eq!(
            v["summary"],
            serde_yaml_ng::Value::String("A package summary".to_string())
        );
        assert!(v.get("depends").is_some());
        assert!(v.get("recommends").is_some());
        assert!(v.get("suggests").is_some());
        assert!(v.get("provides").is_some());
        assert!(v.get("breaks").is_some());
        assert!(v.get("replaces").is_some());
        assert!(v.get("conflicts").is_some());
        assert_eq!(
            v["section"],
            serde_yaml_ng::Value::String("utils".to_string())
        );
        assert_eq!(
            v["multi-arch"],
            serde_yaml_ng::Value::String("same".to_string())
        );
    }

    #[test]
    fn debcraft_package_multi_arch_kebab_case() {
        let pkg = DebcraftPackage {
            multi_arch: Some("foreign".to_string()),
            ..Default::default()
        };
        let v = to_yaml(&pkg);
        assert!(
            v.get("multi-arch").is_some(),
            "multi_arch should serialise as multi-arch"
        );
        assert!(
            v.get("multiArch").is_none(),
            "camelCase must not appear"
        );
        assert!(
            v.get("multi_arch").is_none(),
            "snake_case must not appear in YAML"
        );
    }

    #[test]
    fn debcraft_package_multiple_depends() {
        let pkg = DebcraftPackage {
            depends: vec![
                "libc6 (>= 2.31)".to_string(),
                "libgcc-s1".to_string(),
                "libssl3".to_string(),
            ],
            ..Default::default()
        };
        let v = to_yaml(&pkg);
        let deps = v.get("depends").unwrap();
        assert_eq!(deps[0], serde_yaml_ng::Value::String("libc6 (>= 2.31)".to_string()));
        assert_eq!(deps[1], serde_yaml_ng::Value::String("libgcc-s1".to_string()));
        assert_eq!(deps[2], serde_yaml_ng::Value::String("libssl3".to_string()));
    }

    // Full YAML round-trip: serialise to string and check key formatting.

    #[test]
    fn full_yaml_output_has_kebab_case_keys() {
        let yaml = DebcraftYaml {
            name: "rust-test".to_string(),
            version: "0.1.0-1".to_string(),
            summary: Some("test crate".to_string()),
            description: None,
            maintainer: "Test <test@test.com>".to_string(),
            uploaders: vec![],
            section: Some("rust".to_string()),
            priority: None,
            contact: Some("test@test.com".to_string()),
            source_code: Some("https://github.com/test/test".to_string()),
            vcs_git: Some("https://github.com/test/test.git".to_string()),
            license: Some("MIT OR Apache-2.0".to_string()),
            issues: Some("https://github.com/test/test/issues".to_string()),
            base: Some("ubuntu@24.04".to_string()),
            custom_source_fields: {
                let mut m = BTreeMap::new();
                m.insert("X-Cargo-Crate".to_string(), "test".to_string());
                m
            },
            rules_requires_root: None,
            parts: BTreeMap::new(),
            packages: BTreeMap::new(),
        };
        let output = serde_yaml_ng::to_string(&yaml).unwrap();
        assert!(output.contains("source-code:"), "should have source-code key");
        assert!(output.contains("vcs-git:"), "should have vcs-git key");
        assert!(output.contains("custom-source-fields:"), "should have custom-source-fields key");
        assert!(!output.contains("source_code:"), "should not have snake_case");
        assert!(!output.contains("vcs_git:"), "should not have snake_case");
    }

    #[test]
    fn full_yaml_with_parts_serialises_correctly() {
        let mut parts = BTreeMap::new();
        parts.insert(
            "my-crate".to_string(),
            DebcraftPart {
                plugin: "rust".to_string(),
                source: "https://github.com/example/crate".to_string(),
                rust_channel: Some("stable".to_string()),
                rust_features: vec!["default".to_string(), "full".to_string()],
                build_packages: vec!["libssl-dev".to_string()],
                build_packages_arch: vec![
                    BuildPackageEntry::Simple("librust-foo-dev".to_string()),
                    BuildPackageEntry::WithProfile {
                        package: "librust-bar-dev".to_string(),
                        profiles: vec!["nocheck".to_string()],
                    },
                ],
                after: vec![],
            },
        );

        let yaml = DebcraftYaml {
            name: "rust-crate".to_string(),
            version: "1.0.0-1".to_string(),
            summary: None,
            description: None,
            maintainer: "M <m@m.com>".to_string(),
            uploaders: vec![],
            section: None,
            priority: None,
            contact: None,
            source_code: None,
            vcs_git: None,
            license: None,
            issues: None,
            base: None,
            custom_source_fields: BTreeMap::new(),
            rules_requires_root: None,
            parts,
            packages: BTreeMap::new(),
        };

        let output = serde_yaml_ng::to_string(&yaml).unwrap();
        assert!(output.contains("my-crate:"));
        assert!(output.contains("plugin: rust"));
        assert!(output.contains("rust-channel: stable"));
        assert!(output.contains("rust-features:"));
        assert!(output.contains("build-packages:"));
        assert!(output.contains("build-packages-arch:"));
    }
}
