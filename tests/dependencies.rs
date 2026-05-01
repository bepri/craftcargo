extern crate debcargo;

use std::path::Path;

fn read_manifest(name: &str) -> cargo::core::manifest::Manifest {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(manifest_dir).join("tests").join("dependencies");

    let cargo_toml = crate_dir.join(format!("{name}.toml"));

    let cargo::core::EitherManifest::Real(manifest) = cargo::util::toml::read_manifest(
        &cargo_toml,
        cargo::core::SourceId::for_path(cargo_toml.parent().unwrap()).unwrap(),
        &cargo::GlobalContext::default().unwrap(),
    )
    .unwrap() else {
        panic!("Manifest lacks project and package sections")
    };

    manifest
}

fn read_expected_output(name: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(manifest_dir).join("tests").join("dependencies");

    std::fs::read_to_string(crate_dir.join(format!("{name}.expected"))).unwrap()
}

fn write_expected_output(name: &str, content: &str) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(manifest_dir).join("tests").join("dependencies");

    std::fs::write(crate_dir.join(format!("{name}.expected")), content).unwrap();
}

fn serialize_dependency(dep: &cargo::core::Dependency) -> String {
    let mut output = String::new();

    output.push_str(dep.package_name().as_ref());
    output.push('/');
    output.push_str(&format!("{:?}", dep.version_req()));
    if !dep.features().is_empty() {
        output.push('/');
        output.push_str(&dep.features().join("|"));
    }

    output
}

fn test_crate(name: &str, overwrite: bool) {
    let manifest = read_manifest(name);
    let all_deps = debcargo::crates::all_dependencies_and_features_filtered(&manifest, true);
    let mut actual = String::new();
    for (feature, (other_features, dependencies)) in &all_deps {
        actual.push_str("---\n");
        actual.push_str(&format!("{feature}\n"));
        for f in other_features {
            actual.push_str(&format!("f: {f}\n"));
        }
        for d in dependencies {
            actual.push_str(&format!("d: {}\n", serialize_dependency(d)));
        }
        let (ffs, dds) = debcargo::crates::transitive_deps(&all_deps, feature).unwrap();
        for ff in ffs {
            // Cycle!
            assert_ne!(&ff, feature);
            actual.push_str(&format!("ff: {ff}\n"));
        }

        for dd in dds {
            actual.push_str(&format!("dd: {}\n", serialize_dependency(&dd)));
        }
    }

    let expected = read_expected_output(name);
    if overwrite {
        write_expected_output(name, &actual);
    } else {
        assert_eq!(actual, expected);
    }
}

#[test]
fn debcargo_deps() {
    test_crate("debcargo", false);
}

#[test]
fn gitoxide_deps() {
    test_crate("gitoxide", false);
}

#[test]
fn windows_sys_deps() {
    test_crate("windows-sys", false);
}

#[test]
fn cycle_deps() {
    test_crate("cycle", false);
}
