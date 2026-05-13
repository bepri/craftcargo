extern crate debcargo;

use std::path::Path;

/// Run `debcargo package-debcraft` on a local crate fixture and return the
/// generated `debcraft.yaml` contents, or `None` if the fixture doesn't exist.
fn local_debcraft_test(base_tmpdir: &Path, crate_name: &str, version: &str) -> Option<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(manifest_dir).join("tests").join(crate_name);
    if !crate_dir.exists() {
        return None;
    }

    // Use a per-invocation tempdir so parallel tests don't collide.
    let tmpdir = tempfile::Builder::new()
        .prefix("debcargo-debcraft")
        .tempdir_in(base_tmpdir)
        .expect("Should be able to create temporary directory");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_debcargo"))
        .env("DEBFULLNAME", "Debcargo Test")
        .env("DEBEMAIL", "debcargo@example.com")
        .arg("package-debcraft")
        .arg("--config")
        .arg(format!(
            "{manifest_dir}/tests/{crate_name}-overlay/debian/debcargo.toml"
        ))
        .arg("--directory")
        .arg(tmpdir.path().join("output"))
        .arg("--no-overlay-write-back")
        .arg(crate_name)
        .arg(version)
        .output()
        .expect("Should be able to run `debcargo package-debcraft`");

    assert!(
        output.status.success(),
        "debcargo package-debcraft failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    Some(
        std::fs::read_to_string(tmpdir.path().join("output/debcraft.yaml"))
            .expect("Should be able to read generated debcraft.yaml"),
    )
}

#[test]
fn generate_debcraft_yaml_for_lib_crate() {
    let base_tmpdir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    if let Some(actual) = local_debcraft_test(base_tmpdir, "foobar", "0.1.0") {
        std::fs::write(base_tmpdir.join("foobar.debcraft.actual"), &actual)
            .expect("Should be able to write actual debcraft.yaml");
        let expected = include_str!("foobar.debcraft.expected");
        assert_eq!(actual, expected);
    }
}

#[test]
fn generate_debcraft_yaml_for_semver_crate() {
    let base_tmpdir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    if let Some(actual) = local_debcraft_test(base_tmpdir, "foobar-semver", "0.1.0") {
        std::fs::write(base_tmpdir.join("foobar-semver.debcraft.actual"), &actual)
            .expect("Should be able to write actual debcraft.yaml");
        let expected = include_str!("foobar-semver.debcraft.expected");
        assert_eq!(actual, expected);
    }
}
