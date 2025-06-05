extern crate debcargo;

use debcargo::config::{Config, PackageKey};
use std::path::Path;

#[test]
fn source_package_override() {
    let filepath = Path::new("tests/clap_override.toml");

    let config = Config::parse(filepath).unwrap();

    assert!(config.source.is_some());
    assert_eq!(config.packages.len(), 1);

    assert_eq!(config.policy_version().unwrap(), "4.0.0");

    assert_eq!(config.homepage().unwrap(), "https://clap.rs");

    assert_eq!(config.section(), None);
    assert_eq!(config.build_depends(), None);

    let filepath = Path::new("tests/debcargo_override.toml");
    let config = Config::parse(filepath).unwrap();

    assert!(config.source.is_some());

    assert_eq!(config.section().unwrap(), "rust");

    assert_eq!(config.packages.len(), 2);
    assert_eq!(
        config.package_summary(PackageKey::Bin).unwrap(),
        "Tool to create Debian package from Rust crate"
    );

    assert_eq!(
        config.package_description(PackageKey::Bin).unwrap(),
        "\
This package provides debcargo a tool to create Debian source package from Rust
crate. The package created by this tool is as per the packaging policy set by
Debian Rust team.
"
    );

    let extra = PackageKey::Extra("libblake3-0");
    assert_eq!(config.package_architecture(extra).unwrap(), &vec!["any"]);
    assert_eq!(config.package_section(extra).unwrap(), "libs");
    assert_eq!(
        config.package_depends(extra).unwrap(),
        &vec!["${misc:Depends}", "${shlibs:Depends}"]
    );
    assert_eq!(
        config.package_description(extra).unwrap(),
        "BLAKE3 hash function - C library"
    );
}

#[test]
fn binary_package_override() {
    let filepath = Path::new("tests/tiny-dfr_override.toml");
    let config = Config::parse(filepath).unwrap();

    assert_eq!(config.packages.len(), 1);

    assert_eq!(
        config.package_summary(PackageKey::Bin),
        Some("dynamic touch bar daemon")
    );

    assert_eq!(
        config.package_description(PackageKey::Bin),
        Some(
            "\
    This package contains tiny-dfr, the userland touch bar daemon.

tiny-dfr shows the function row and media control keys (brightness,
volume, backlight, play, etc) on your touch bar. Currently supported
platforms are Apple Silicon and T2 Macs.
"
        )
    );

    assert_eq!(
        config.package_architecture(PackageKey::Bin),
        Some(&vec!["arm64".to_string(), "amd64".to_string()])
    );
}

#[test]
fn sd_top_level() {
    let filepath = Path::new("tests/debcargo_override_top_level.toml");
    let config = Config::parse(filepath).unwrap();

    assert!(config.source.is_some());

    assert_eq!(config.section().unwrap(), "rust");

    assert_eq!(
        config.summary.unwrap(),
        "Tool to create Debian package from Rust crate"
    );
    assert_eq!(
        config.description.unwrap(),
        "\
This package provides debcargo a tool to create Debian source package from Rust
crate. The package created by this tool is as per the packaging policy set by
Debian Rust team.
"
    );
}

#[test]
fn unknown_fields_captured_with_warning() {
    let file_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("test_debcargo.toml");

    // Test unknown field at top level
    std::fs::write(
        &file_path,
        br#"
semver_suffix = true
overlay = "."
verlay = "typo"  # This should be captured as an unknown field
"#,
    )
    .unwrap();

    let config = Config::parse(&file_path).unwrap();
    assert!(config.unknown_fields.contains_key("verlay"));

    // Test unknown field in source section
    std::fs::write(
        &file_path,
        r#"
[source]
section = "rust"
unknown_field = "value"
"#,
    )
    .unwrap();

    let config = Config::parse(&file_path).unwrap();
    let source = config.source.unwrap();
    assert!(source.unknown_fields.contains_key("unknown_field"));

    // Test unknown field in packages section
    std::fs::write(
        &file_path,
        r#"
[packages.lib]
section = "libs"
unknwon_field = "value"
"#,
    )
    .unwrap();

    let config = Config::parse(&file_path).unwrap();
    let lib_package = config.packages.get("lib").unwrap();
    assert!(lib_package.unknown_fields.contains_key("unknwon_field"));
}
