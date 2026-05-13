extern crate debcargo;

use std::path::Path;

fn local_package_test(tmpdir: &Path, crate_name: &str, version: &str) -> Option<String> {
    let tempdir = tempfile::Builder::new()
        .prefix("debcargo")
        .tempdir_in(tmpdir)
        .expect("Should be able to create temporary directory");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(manifest_dir).join("tests").join(crate_name);
    if !crate_dir.exists() {
        return None;
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_debcargo"))
        .env("DEBFULLNAME", "Debcargo Test")
        .env("DEBEMAIL", "debcargo@example.com")
        .arg("package")
        .arg("--config")
        .arg(format!(
            "{manifest_dir}/tests/{crate_name}-overlay/debian/debcargo.toml"
        ))
        .arg("--directory")
        .arg(tempdir.path().join("output"))
        .arg("--no-overlay-write-back")
        .arg(crate_name)
        .arg(version)
        .output()
        .expect("Should be able to run `debcargo package`");

    assert!(output.status.success());

    Some(
        std::fs::read_to_string(tempdir.path().join("output/debian/control"))
            .expect("Should be able to read generated debian/control file"),
    )
}

#[test]
fn generate_package_with_crate_src() {
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    if let Some(actual) = local_package_test(out_dir, "foobar", "0.1.0") {
        std::fs::write(out_dir.join("foobar.actual"), &actual)
            .expect("Should be able to write out generate control contents");
        let expected = include_str!("foobar.expected");
        assert_eq!(actual, expected);
    }
}

#[test]
fn generate_package_with_semver_crate_src() {
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    if let Some(actual) = local_package_test(out_dir, "foobar-semver", "0.1.0") {
        std::fs::write(out_dir.join("foobar-semver.actual"), &actual)
            .expect("Should be able to write out generate control contents");
        let expected = include_str!("foobar-semver.expected");
        assert_eq!(actual, expected);
    }
}
