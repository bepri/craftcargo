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
}
