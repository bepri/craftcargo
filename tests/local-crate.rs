extern crate debcargo;

use std::path::PathBuf;

use debcargo::package::{PackageExecuteArgs, PackageExtractArgs, PackageInitArgs, PackageProcess};

#[test]
fn generate_package_with_crate_src() {
    let tempdir = tempfile::Builder::new()
        .prefix("debcargo")
        .tempdir_in(".")
        .expect("Should be able to create temporary directory");
    let init_args = PackageInitArgs {
        crate_name: "foobar".to_string(),
        version: Some("0.1.0".to_string()),
        config: Some(PathBuf::from("tests/foobar-overlay/debian/debcargo.toml")),
    };
    let extract_args = PackageExtractArgs {
        directory: Some(tempdir.path().join("output").to_owned()),
    };
    let execute_args = PackageExecuteArgs {
        changelog_ready: false,
        copyright_guess_harder: false,
        no_overlay_write_back: true,
    };
    let mut process = PackageProcess::init(init_args).expect("SHould be able to init packaging");
    process
        .extract(extract_args)
        .expect("Should be able to extract crate");
    process
        .apply_overrides()
        .expect("Should be able to apply overrides");
    process
        .prepare_orig_tarball()
        .expect("Should be able to prepare orig tarball");
    process
        .prepare_debian_folder(execute_args)
        .expect("Should be able to prepare debian/ dir");
    process
        .post_package_checks()
        .expect("Post-package-checks shouldn't fail");

    let control = std::fs::read_to_string(tempdir.path().join("output/debian/control"))
        .expect("Should be able to read generated debian/control file");

    eprintln!("{control}");

    let expected = include_str!("foobar.expected");
    assert_eq!(control, expected);
}
