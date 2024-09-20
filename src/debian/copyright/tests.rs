use super::{debian_copyright, get_licenses};

use std::{io::Write, path::Path};

use cargo::{
    core::{package::Package, EitherManifest, SourceId},
    GlobalContext,
};
use toml::toml;

#[test]
fn check_get_licenses() {
    let test_data: &[(&str, &[(&str, bool)])] = &[
        ("AGPL-3.0", &[("AGPL-3.0", true)]),
        ("AcmeCorp-1.0", &[("AcmeCorp-1.0", false)]),
        ("AGPL-3.0-or-later", &[("AGPL-3.0-or-later", true)]),
        ("Apache-2.0/MIT", &[("Apache-2.0", true), ("MIT", true)]),
        ("Apache-2.0 or MIT", &[("Apache-2.0", true), ("MIT", true)]),
        (
            "FooBar-1.0 AND MIT",
            &[("FooBar-1.0", false), ("MIT", true)],
        ),
    ];
    for (name, expected) in test_data {
        let licenses = get_licenses(name).expect("getting licenses failed");
        let found: Vec<_> = licenses
            .iter()
            .map(|l| (l.name.as_str(), !l.text.starts_with("FIXME")))
            .collect();
        assert_eq!(&found[..], &expected[..]);
    }
}

#[test]
fn check_debian_copyright_authors() {
    let checks = vec![
        (
            vec![],
            vec!["FIXME (overlay) UNKNOWN-AUTHORS FIXME (overlay) UNKNOWN-YEARS"],
        ),
        (
            vec!["Jordan Doe"],
            vec!["FIXME (overlay) UNKNOWN-YEARS Jordan Doe"],
        ),
        (
            vec!["Jordan Doe", "Jane Doe"],
            vec![
                "FIXME (overlay) UNKNOWN-YEARS Jordan Doe",
                "FIXME (overlay) UNKNOWN-YEARS Jane Doe",
            ],
        ),
    ];

    for (input, expected_output) in checks.into_iter() {
        let package = build_package_with_authors(input);
        let srcdir = tempfile::tempdir().unwrap();
        let copyright = debian_copyright(
            srcdir.path(),
            package.manifest(),
            package.manifest_path(),
            "Jordan Doe",
            &[],
            (2000, 2020),
            false,
        )
        .unwrap();
        let mut generated = false;
        for file in &copyright.files {
            if file.files == "*" {
                assert_eq!(file.copyright, expected_output);
                generated = true;
            }
        }
        assert!(generated);
    }
}

fn build_package_with_authors(authors: Vec<&str>) -> Package {
    let authors: Vec<String> = authors.into_iter().map(|s| s.to_string()).collect();
    let toml = toml! {
        [package]
        name = "mypackage"
        version = "1.2.3"
        authors = authors
        license = "AGPLv3"
    };
    let tmp_dir = tempfile::tempdir().unwrap();
    let manifest_path = tmp_dir.path().join("Cargo.toml");
    let mut manifest_file = std::fs::File::create(&manifest_path).unwrap();
    manifest_file
        .write_all(toml::to_string(&toml).unwrap().as_bytes())
        .unwrap();

    let src_dir = tmp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::File::create(src_dir.join("lib.rs")).unwrap();
    #[cfg(unix)]
    let package_root = Path::new("/");
    #[cfg(windows)]
    let package_root = Path::new("C:\\");
    let source_id = SourceId::for_path(package_root).unwrap();
    let context = GlobalContext::default().unwrap();
    let manifest = cargo::util::toml::read_manifest(&manifest_path, source_id, &context).unwrap();
    if let EitherManifest::Real(manifest) = manifest {
        Package::new(manifest, Path::new("/path/to/manifest"))
    } else {
        unimplemented!();
    }
}
