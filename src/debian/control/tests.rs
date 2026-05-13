use super::{
    base_deb_name, deb_feature_name, deb_name, deb_upstream_version, dsc_name, Description,
    Package, PkgTest, PkgTestRestriction, Source,
};
use crate::{
    config::{Config, SourceOverride},
    debian::BuildDeps,
};
use semver::{BuildMetadata, Prerelease, Version};

#[test]
fn source_to_string() {
    let instance = Source::new(
        "rsa",
        None,
        "rsa",
        "1.2.3",
        "https://github.com/RustCrypto/RSA",
        true,
        "Jelmer Vernooĳ <jelmer@debian.org>".to_owned(),
        vec!["Jelmer Vernooĳ <jelmer@debian.org>".to_owned()],
        BuildDeps::default(),
        None,
    )
    .unwrap();

    let expected = "Source: rust-rsa\nSection: rust\nMaintainer: Jelmer Vernooĳ <jelmer@debian.org>\nUploaders:\n Jelmer Vernooĳ <jelmer@debian.org>\nStandards-Version: 4.7.3\nVcs-Git: https://salsa.debian.org/rust-team/debcargo-conf.git [src/rsa]\nVcs-Browser: https://salsa.debian.org/rust-team/debcargo-conf/tree/master/src/rsa\nHomepage: https://github.com/RustCrypto/RSA\nX-Cargo-Crate: rsa\nX-Cargo-Crate-Version: 1.2.3\n";

    assert_eq!(expected, instance.to_string());
}

fn empty_source_overrides() -> SourceOverride {
    SourceOverride::new(
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(vec!["rust-digest".to_owned()]),
        Some(vec![String::new()]),
        Some(vec![String::new()]),
        Some(vec!["rust-const-oid".to_owned()]),
        Some(false),
    )
}

#[test]
fn test_description_display() {
    let instance = Description::new("prefix".to_owned(), "suffix".to_owned());

    assert_eq!("prefixsuffix", instance.to_string());
}

#[test]
fn test_apply_overrides() {
    let mut instance = Source::new(
        "rsa",
        None,
        "rsa",
        "5.1.2",
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
        Some("no".to_owned()),
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

    let config = Config {
        source: Some(empty_source_overrides()),
        ..Config::default()
    };
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
        pre: Prerelease::default(),
        build: BuildMetadata::default(),
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
        &f_deps,
        o_deps,
        &f_provides,
        &f_recommends,
        &f_suggests,
        None,
    );

    assert!(instance.is_ok());
    let instance = instance.unwrap();
    assert_eq!("any", instance.arch);
    assert_eq!(Some("same".to_string()), instance.multi_arch);
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
    assert_eq!(None, instance.multi_arch);
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
    let summary: Description = Description::new(String::new(), String::new());
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        String::new(),
    );
    let instance = Package::new_bin(basename, name_suffix, section, summary, description);

    let expected = "Package: rsa\nArchitecture: any\nSection: rust\nDepends:\n ${misc:Depends},\n ${shlibs:Depends},\n ${cargo:Depends}\nRecommends:\n ${cargo:Recommends}\nSuggests:\n ${cargo:Suggests}\nProvides:\n ${cargo:Provides}\nBuilt-Using: ${cargo:Built-Using}\nStatic-Built-Using: ${cargo:Static-Built-Using}\nDescription: \n description_start\n .\n empty lines\n .\n description_stop\n";

    assert_eq!(expected, instance.to_string());
}

#[test]
fn test_package_display_arch() {
    let basename: &str = "tiny-dfr";
    let name_suffix: Option<&str> = None;
    let section = Some("utils");
    let summary: Description = Description::new(String::new(), String::new());
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        String::new(),
    );
    let mut instance = Package::new_bin(basename, name_suffix, section, summary, description);
    instance.arch = "arm64 amd64".to_string();

    let expected = "Package: tiny-dfr\nArchitecture: arm64 amd64\nSection: utils\nDepends:\n ${misc:Depends},\n ${shlibs:Depends},\n ${cargo:Depends}\nRecommends:\n ${cargo:Recommends}\nSuggests:\n ${cargo:Suggests}\nProvides:\n ${cargo:Provides}\nBuilt-Using: ${cargo:Built-Using}\nStatic-Built-Using: ${cargo:Static-Built-Using}\nDescription: \n description_start\n .\n empty lines\n .\n description_stop\n";

    assert_eq!(expected, instance.to_string());
}

#[test]
fn test_package_summary_check_len() {
    let basename: &str = "rsa";
    let name_suffix: Option<&str> = None;
    let section = Some("rust");
    let summary: Description = Description::new(String::new(), String::new());
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        String::new(),
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
        String::new(),
    );
    let description: Description = Description::new(
        "description_start\n\nempty lines\n\ndescription_stop".to_owned(),
        String::new(),
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
    let extra_restricts: Vec<_> = vec![];
    let architecture: Vec<&str> = vec![];
    let instance = PkgTest::new(
        name,
        crate_name,
        feature,
        version,
        &extra_test_args,
        &depends,
        &extra_restricts,
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
        pre: Prerelease::default(),
        build: BuildMetadata::default(),
    };

    let result = deb_upstream_version(&version, None);

    let expected = "0.9.7";

    assert_eq!(expected, result);

    let result = deb_upstream_version(&version, Some("dfsg9"));

    let expected = "0.9.7+dfsg9";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_upstream_version_with_pre() {
    let version = Version {
        major: 0,
        minor: 9,
        patch: 7,
        pre: Prerelease::new("alpha").unwrap(),
        build: BuildMetadata::default(),
    };

    let result = deb_upstream_version(&version, Some("dfsg9"));

    let expected = "0.9.7~alpha+dfsg9";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_upstream_version_stable_release() {
    let version = Version::new(1, 0, 0);
    assert_eq!("1.0.0", deb_upstream_version(&version, None));
}

#[test]
fn test_deb_upstream_version_large_numbers() {
    let version = Version::new(123, 456, 789);
    assert_eq!("123.456.789", deb_upstream_version(&version, None));
}

#[test]
fn test_deb_upstream_version_pre_without_repack() {
    let version = Version {
        major: 2,
        minor: 0,
        patch: 0,
        pre: Prerelease::new("beta.1").unwrap(),
        build: BuildMetadata::default(),
    };
    assert_eq!("2.0.0~beta.1", deb_upstream_version(&version, None));
}

#[test]
fn test_deb_upstream_version_pre_rc() {
    let version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        pre: Prerelease::new("rc.1").unwrap(),
        build: BuildMetadata::default(),
    };
    assert_eq!("1.0.0~rc.1", deb_upstream_version(&version, None));
}

#[test]
fn test_deb_upstream_version_repack_dfsg() {
    let version = Version::new(3, 2, 1);
    assert_eq!("3.2.1+dfsg", deb_upstream_version(&version, Some("dfsg")));
}

#[test]
fn test_deb_upstream_version_repack_custom_suffix() {
    let version = Version::new(1, 5, 0);
    assert_eq!("1.5.0+ds1", deb_upstream_version(&version, Some("ds1")));
}

#[test]
fn test_base_deb_name() {
    let result = base_deb_name("derive_more");

    let expected = "derive-more";

    assert_eq!(expected, result);
}

#[test]
fn test_base_deb_name_already_lowercase() {
    assert_eq!("serde", base_deb_name("serde"));
}

#[test]
fn test_base_deb_name_uppercase() {
    assert_eq!("openssl", base_deb_name("OpenSSL"));
}

#[test]
fn test_base_deb_name_multiple_underscores() {
    assert_eq!("my-cool-crate", base_deb_name("my_cool_crate"));
}

#[test]
fn test_base_deb_name_mixed_case_and_underscores() {
    assert_eq!("tokio-util", base_deb_name("Tokio_Util"));
}

#[test]
fn test_base_deb_name_hyphens_preserved() {
    assert_eq!("serde-json", base_deb_name("serde-json"));
}

#[test]
fn test_base_deb_name_single_char() {
    assert_eq!("x", base_deb_name("x"));
}

#[test]
fn test_base_deb_name_numbers() {
    assert_eq!("sha2", base_deb_name("sha2"));
}

#[test]
fn test_base_deb_name_numeric_crate() {
    assert_eq!("blake3", base_deb_name("blake3"));
}

#[test]
fn test_dsc_name() {
    let result = dsc_name("derive_more");

    let expected = "rust-derive-more";

    assert_eq!(expected, result);
}

#[test]
fn test_dsc_name_simple() {
    assert_eq!("rust-serde", dsc_name("serde"));
}

#[test]
fn test_dsc_name_with_hyphens() {
    assert_eq!("rust-serde-json", dsc_name("serde-json"));
}

#[test]
fn test_dsc_name_uppercase_normalised() {
    assert_eq!("rust-openssl", dsc_name("OpenSSL"));
}

#[test]
fn test_deb_name() {
    let result = deb_name("derive_more");

    let expected = "librust-derive-more-dev";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_name_simple() {
    assert_eq!("librust-serde-dev", deb_name("serde"));
}

#[test]
fn test_deb_name_with_hyphens() {
    assert_eq!("librust-serde-json-dev", deb_name("serde-json"));
}

#[test]
fn test_deb_name_uppercase_normalised() {
    assert_eq!("librust-openssl-dev", deb_name("OpenSSL"));
}

#[test]
fn test_deb_name_underscores() {
    assert_eq!("librust-proc-macro2-dev", deb_name("proc_macro2"));
}

#[test]
fn test_deb_feature_name() {
    let result = deb_feature_name("derive_more", "add");

    let expected = "librust-derive-more+add-dev";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_feature_name_simple() {
    assert_eq!(
        "librust-serde+derive-dev",
        deb_feature_name("serde", "derive")
    );
}

#[test]
fn test_deb_feature_name_with_underscore_feature() {
    assert_eq!("librust-serde+std-dev", deb_feature_name("serde", "std"));
}

#[test]
fn test_deb_feature_name_complex_feature() {
    assert_eq!("librust-tokio+full-dev", deb_feature_name("tokio", "full"));
}

#[test]
fn test_deb_feature_name_underscore_in_feature() {
    assert_eq!(
        "librust-regex+unicode-perl-dev",
        deb_feature_name("regex", "unicode_perl")
    );
}

#[test]
fn test_deb_feature_name_uppercase_crate() {
    assert_eq!(
        "librust-openssl+vendored-dev",
        deb_feature_name("OpenSSL", "vendored")
    );
}

struct PkgTestFmtData<'a> {
    feature: &'a str,
    extra_test_args: Vec<&'a str>,
    depends: Vec<String>,
    extra_restricts: Vec<PkgTestRestriction>,
    architecture: &'a [&'a str],
}

#[test]
fn pkgtest_fmt_has_expected_output() {
    let checks = vec![
        (
            PkgTestFmtData {
                feature: "",
                extra_test_args: Vec::new(),
                depends: Vec::new(),
                extra_restricts: vec![PkgTestRestriction::AllowStderr],
                architecture: &[],
            },
            "Test-Command: /usr/share/cargo/bin/cargo-auto-test crate 1.0 --all-targets\nFeatures: test-name=librust-crate-dev:\nDepends: dh-cargo (>= 33~), @\nRestrictions: allow-stderr\n"
,
        ),
        (
            PkgTestFmtData {
                feature: "X",
                extra_test_args: vec!["--no-default-features", "--features X"],
                depends: vec!["libfoo-dev".into(), "bar".into()],
                extra_restricts: vec![PkgTestRestriction::AllowStderr, PkgTestRestriction::Flaky],
                architecture: &["!riscv64"],
            },
            "Test-Command: /usr/share/cargo/bin/cargo-auto-test crate 1.0 --all-targets --no-default-features --features X\nFeatures: test-name=librust-crate-dev:X\nDepends: dh-cargo (>= 33~), libfoo-dev, bar, @\nRestrictions: allow-stderr, flaky\nArchitecture: !riscv64\n",
        ),
    ];

    for (check, expected) in checks {
        let pkgtest = PkgTest::new(
            "librust-crate-dev",
            "crate",
            check.feature,
            "1.0",
            &check.extra_test_args,
            &check.depends,
            &check.extra_restricts,
            check.architecture,
        )
        .unwrap();

        let output = pkgtest.to_string();
        assert_eq!(output, expected);

        for ln in output.lines() {
            let trimmed = ln.trim_end();
            assert_eq!(trimmed, ln);
        }
    }
}
