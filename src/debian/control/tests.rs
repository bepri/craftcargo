use super::{
    base_deb_name, deb_feature_name, deb_name, deb_upstream_version, dsc_name, Description,
    Package, PkgTest, Source,
};
use crate::{
    config::{Config, SourceOverride},
    debian::BuildDeps,
};
use semver::{Prerelease, Version};

#[test]
fn source_to_string() {
    let instance = Source::new(
        "rsa",
        None,
        "rsa",
        "https://github.com/RustCrypto/RSA",
        true,
        "Jelmer Vernooĳ <jelmer@debian.org>".to_owned(),
        vec!["Jelmer Vernooĳ <jelmer@debian.org>".to_owned()],
        BuildDeps::default(),
        "no".to_owned(),
    )
    .unwrap();

    let expected = "Source: rust-rsa\nSection: rust\nPriority: optional\nMaintainer: Jelmer Vernooĳ <jelmer@debian.org>\nUploaders:\n Jelmer Vernooĳ <jelmer@debian.org>\nStandards-Version: 4.7.0\nVcs-Git: https://salsa.debian.org/rust-team/debcargo-conf.git [src/rsa]\nVcs-Browser: https://salsa.debian.org/rust-team/debcargo-conf/tree/master/src/rsa\nHomepage: https://github.com/RustCrypto/RSA\nX-Cargo-Crate: rsa\nRules-Requires-Root: no\n";

    assert_eq!(expected, instance.to_string());
}

fn empty_source_overrides() -> SourceOverride {
    SourceOverride::new(
        Some("".to_owned()),
        Some("".to_owned()),
        Some("".to_owned()),
        Some("".to_owned()),
        Some("".to_owned()),
        Some(vec!["rust-digest".to_owned()]),
        Some(vec!["rust-const-oid".to_owned()]),
    )
}

#[test]
fn test_description_display() {
    let instance = Description::new("prefix".to_owned(), "suffix".to_owned());

    assert_eq!("prefixsuffix", instance.to_string())
}

#[test]
fn test_apply_overrides() {
    let mut instance = Source::new(
        "rsa",
        None,
        "rsa",
        "https://github.com/RustCrypto/RSA",
        true,
        "Jelmer Vernooĳ <jelmer@debian.org>".to_owned(),
        vec!["Jelmer Vernooĳ <jelmer@debian.org>".to_owned()],
        BuildDeps {
            build_depends: vec![
                "rust-const-oid".to_owned(),
                "rust-num-bigint-dig".to_owned(),
            ],
            ..BuildDeps::default()
        },
        "no".to_owned(),
    )
    .unwrap();

    assert_eq!("rust", instance.section);
    assert_eq!(
        vec!["rust-const-oid", "rust-num-bigint-dig"],
        instance.build_deps.build_depends
    );
    assert_eq!("https://github.com/RustCrypto/RSA", instance.homepage);
    assert_eq!(
        "https://salsa.debian.org/rust-team/debcargo-conf.git [src/rsa]",
        instance.vcs_git
    );
    assert_eq!(
        "https://salsa.debian.org/rust-team/debcargo-conf/tree/master/src/rsa",
        instance.vcs_browser
    );

    let mut config = Config::default();
    config.source = Some(empty_source_overrides());
    instance.apply_overrides(&config);

    assert_eq!("", instance.section);
    assert_eq!(
        vec!["rust-num-bigint-dig", "rust-digest"],
        instance.build_deps.build_depends
    );
    assert_eq!("", instance.homepage);
    assert_eq!("", instance.vcs_git);
    assert_eq!("", instance.vcs_browser);
}

#[test]
fn test_package_new() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let version: &Version = &Version {
        major: 0,
        minor: 9,
        patch: 7,
        pre: Default::default(),
        build: Default::default(),
    };
    let summary: Description = Description::new("summary_pre".to_owned(), "summary_suf".to_owned());
    let description: Description =
        Description::new("description_pre".to_owned(), "description_suf".to_owned());
    let feature: Option<&str> = None;
    let f_deps: Vec<&str> = vec![];
    let o_deps: Vec<String> = vec![];
    let f_provides: Vec<&str> = vec![];
    let f_recommends: Vec<&str> = vec![];
    let f_suggests: Vec<&str> = vec![];
    let instance = Package::new(
        basename,
        name_suffix,
        version,
        summary,
        description,
        feature,
        f_deps,
        o_deps,
        f_provides,
        f_recommends,
        f_suggests,
    );

    assert!(instance.is_ok());
    let instance = instance.unwrap();
    assert_eq!("any", instance.arch);
    assert_eq!("same", instance.multi_arch);
    assert_eq!(None, instance.section);
    assert_eq!(vec!["${misc:Depends}"], instance.depends);
    assert_eq!(Vec::<String>::new(), instance.recommends);
    assert_eq!(Vec::<String>::new(), instance.suggests);
    assert_eq!(
        vec![
            "librust-rsa-0-dev (= ${binary:Version})",
            "librust-rsa-0.9-dev (= ${binary:Version})",
            "librust-rsa-0.9.7-dev (= ${binary:Version})"
        ],
        instance.provides
    );
    assert_eq!("summary_pre", instance.summary.prefix);
    assert_eq!("summary_suf", instance.summary.suffix);
    assert_eq!("description_pre", instance.description.prefix);
    assert_eq!("description_suf", instance.description.suffix);
    assert_eq!(Vec::<String>::new(), instance.extra_lines);
}

#[test]
fn test_package_new_bin() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let section = Some("rust");
    let summary: Description = Description::new("summary_pre".to_owned(), "summary_suf".to_owned());
    let description: Description =
        Description::new("description_pre".to_owned(), "description_suf".to_owned());
    let instance = Package::new_bin(basename, name_suffix, section, summary, description);

    assert_eq!("any", instance.arch);
    assert_eq!("allowed", instance.multi_arch);
    assert_eq!(Some("rust".to_owned()), instance.section);
    assert_eq!(
        vec!["${misc:Depends}", "${shlibs:Depends}", "${cargo:Depends}"],
        instance.depends
    );
    assert_eq!(vec!["${cargo:Recommends}"], instance.recommends);
    assert_eq!(vec!["${cargo:Suggests}"], instance.suggests);
    assert_eq!(vec!["${cargo:Provides}"], instance.provides);
    assert_eq!("summary_pre", instance.summary.prefix);
    assert_eq!("summary_suf", instance.summary.suffix);
    assert_eq!("description_pre", instance.description.prefix);
    assert_eq!("description_suf", instance.description.suffix);
    assert_eq!(
        vec![
            "Built-Using: ${cargo:Built-Using}",
            "Static-Built-Using: ${cargo:Static-Built-Using}"
        ],
        instance.extra_lines
    );
}

#[test]
fn test_package_display() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let section = Some("rust");
    let summary: Description = Description::new("".to_owned(), "".to_owned());
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        "".to_owned(),
    );
    let instance = Package::new_bin(basename, name_suffix, section, summary, description);

    let expected = "Package: rsa\nArchitecture: any\nMulti-Arch: allowed\nSection: rust\nDepends:\n ${misc:Depends},\n ${shlibs:Depends},\n ${cargo:Depends}\nRecommends:\n ${cargo:Recommends}\nSuggests:\n ${cargo:Suggests}\nProvides:\n ${cargo:Provides}\nBuilt-Using: ${cargo:Built-Using}\nStatic-Built-Using: ${cargo:Static-Built-Using}\nDescription: \n description_start\n .\n empty lines\n .\n description_stop\n";

    assert_eq!(expected, instance.to_string());
}

#[test]
fn test_package_summary_check_len() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let section = Some("rust");
    let summary: Description = Description::new("".to_owned(), "".to_owned());
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        "".to_owned(),
    );
    let instance = Package::new_bin(basename, name_suffix, section, summary, description);

    assert!(instance.summary_check_len().is_ok());
}

#[test]
fn test_package_summary_check_len_err() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let section = Some("rust");
    // 81 character long line
    let summary: Description = Description::new(
        "123456789012345678901234567890123456789012345678901234567890123456789012345678901"
            .to_owned(),
        "".to_owned(),
    );
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        "".to_owned(),
    );
    let instance = Package::new_bin(basename, name_suffix, section, summary, description);

    assert!(instance.summary_check_len().is_err());
}

#[test]
fn test_pkg_test_new() {
    let name = "rsa";
    let crate_name = "rsa";
    let feature = "pem";
    let version = "0.9.7";
    let extra_test_args: Vec<&str> = vec![];
    let depends: Vec<String> = vec![];
    let extra_restricts: Vec<&str> = vec![];
    let architecture: Vec<&str> = vec![];
    let instance = PkgTest::new(
        name,
        crate_name,
        feature,
        version,
        extra_test_args,
        &depends,
        extra_restricts,
        &architecture,
    );

    assert!(instance.is_ok());
    let instance = instance.unwrap();
    assert_eq!("rsa", instance.name);
    assert_eq!("rsa", instance.crate_name);
    assert_eq!("pem", instance.feature);
    assert_eq!("0.9.7", instance.version);
    assert_eq!(Vec::<String>::new(), instance.extra_test_args);
    assert_eq!(Vec::<String>::new(), instance.depends);
    assert_eq!(Vec::<String>::new(), instance.extra_restricts);
    assert_eq!(Vec::<String>::new(), instance.architecture);
}

#[test]
fn test_deb_upstream_version_without_pre() {
    let version = Version {
        major: 0,
        minor: 9,
        patch: 7,
        pre: Default::default(),
        build: Default::default(),
    };

    let result = deb_upstream_version(&version);

    let expected = "0.9.7";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_upstream_version_with_pre() {
    let version = Version {
        major: 0,
        minor: 9,
        patch: 7,
        pre: Prerelease::new("alpha").unwrap(),
        build: Default::default(),
    };

    let result = deb_upstream_version(&version);

    let expected = "0.9.7~alpha";

    assert_eq!(expected, result);
}

#[test]
fn test_base_deb_name() {
    let result = base_deb_name("derive_more");

    let expected = "derive-more";

    assert_eq!(expected, result);
}

#[test]
fn test_dsc_name() {
    let result = dsc_name("derive_more");

    let expected = "rust-derive-more";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_name() {
    let result = deb_name("derive_more");

    let expected = "librust-derive-more-dev";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_feature_name() {
    let result = deb_feature_name("derive_more", "add");

    let expected = "librust-derive-more+add-dev";

    assert_eq!(expected, result);
}

struct PkgTestFmtData<'a> {
    feature: &'a str,
    extra_test_args: Vec<&'a str>,
    depends: Vec<String>,
    extra_restricts: Vec<&'a str>,
    architecture: &'a [&'a str],
}

#[test]
fn pkgtest_fmt_has_no_extra_whitespace() {
    let checks = vec![
        PkgTestFmtData {
            feature: "",
            extra_test_args: Vec::new(),
            depends: Vec::new(),
            extra_restricts: Vec::new(),
            architecture: &[],
        },
        PkgTestFmtData {
            feature: "X",
            extra_test_args: vec!["--no-default-features", "--features X"],
            depends: vec!["libfoo-dev".into(), "bar".into()],
            extra_restricts: vec!["flaky"],
            architecture: &["!riscv64"],
        },
    ];

    for check in checks {
        let pkgtest = PkgTest::new(
            "librust-crate-dev",
            "crate",
            check.feature,
            "1.0",
            check.extra_test_args,
            &check.depends,
            check.extra_restricts,
            check.architecture,
        )
        .unwrap();

        for ln in pkgtest.to_string().lines() {
            let trimmed = ln.trim_end();
            assert_eq!(trimmed, ln);
        }
    }
}
