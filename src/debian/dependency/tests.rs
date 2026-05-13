use crate::debian::deb_dep_add_nocheck;
use crate::debian::dependency::{VRange, V};

#[test]
fn test_v_new_mmp() {
    let v = V::new(&semver::Comparator::parse("0.9.7").unwrap()).unwrap();

    assert_eq!((0, 9, 7), v.mmp());
}

#[test]
fn test_v_new_inclast() {
    let v = V::new(&semver::Comparator::parse("0.9.7").unwrap()).unwrap();

    assert_eq!((0, 9, 8), v.inclast().mmp());
}

#[test]
fn test_v_neq() {
    let v1 = V::new(&semver::Comparator::parse("0.9.7").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.9.8").unwrap()).unwrap();

    assert!(v1 != v2);
}

#[test]
fn test_v_eq() {
    let v1 = V::new(&semver::Comparator::parse("0.10.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    assert!(v1 == v2);
}

#[test]
fn test_v_display() {
    let v1 = V::new(&semver::Comparator::parse("0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();
    let v3 = V::new(&semver::Comparator::parse("0.10.0").unwrap()).unwrap();

    assert_eq!("0", v1.to_string());
    assert_eq!("0.10", v2.to_string());
    assert_eq!("0.10.0", v3.to_string());
}

#[test]
fn test_v_range() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    let mut vr = VRange::new();
    assert!(vr.ge.is_none());
    assert!(vr.lt.is_none());

    vr.constrain_ge(v1.clone());
    assert!(vr.ge.is_some());
    assert!(vr.ge.as_ref().unwrap() == &v1);
    assert!(vr.ge.as_ref().unwrap() < &v2);
    assert!(vr.lt.is_none());

    vr.constrain_lt(v2.clone());
    assert!(vr.ge.is_some());
    assert!(vr.lt.is_some());
    assert!(vr.ge.as_ref().unwrap() == &v1);
    assert!(vr.lt.as_ref().unwrap() == &v2);
}

#[test]
fn test_v_range_to_deb_clause_empty() {
    let vr = VRange::new();

    assert_eq!(
        vec!["base+feature"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_ge_only() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);

    assert_eq!(
        vec!["base+feature (>= 0.9)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_lt_only() {
    let v1 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_lt(v1);

    assert_eq!(
        vec!["base+feature (<< 0.10)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_minor_one_apart() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base-0.9+feature"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_minor_two_apart() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.11").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 0.9)", "base+feature (<< 0.11)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_minor_complicated_apart() {
    let v1 = V::new(&semver::Comparator::parse(">= 0.1.2").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.4").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 0.1.2)", "base+feature (<< 0.4)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_minor_complicated() {
    let v1 = V::new(&semver::Comparator::parse(">= 0.1.2").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.1.5").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base-0.1+feature (>= 0.1.2)", "base-0.1+feature (<< 0.1.5)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_minor_more_complicated_apart() {
    let v1 = V::new(&semver::Comparator::parse(">= 0.1.2").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.4.5").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 0.1.2)", "base+feature (<< 0.4.5)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_major_one_apart() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("10.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base-9+feature"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_major_two_apart() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("11.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 9.0)", "base+feature (<< 11.0)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_major_complicated_apart() {
    let v1 = V::new(&semver::Comparator::parse(">= 1.2.3").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("4.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 1.2.3)", "base+feature (<< 4.0)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_major_complicated() {
    let v1 = V::new(&semver::Comparator::parse(">= 1.2.3").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("1.9.9").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base-1+feature (>= 1.2.3)", "base-1+feature (<< 1.9.9)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_major_more_complicated_apart() {
    let v1 = V::new(&semver::Comparator::parse(">= 1.2.3").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("4.5.6").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base+feature (>= 1.2.3)", "base+feature (<< 4.5.6)"],
        vr.to_deb_clause("base", "+feature").unwrap()
    );
}

#[test]
fn test_v_range_to_deb_clause_bad_range() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("11.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v2);
    vr.constrain_lt(v1);

    let err = vr.to_deb_clause("base", "+feature");
    assert!(err.is_err());
    assert_eq!(
        "bad version range: >= 11.0, << 9.0",
        err.unwrap_err().to_string()
    );
}

#[test]
fn test_deb_dep_add_nocheck() {
    let result = deb_dep_add_nocheck(" librust-sequoia-policy-config-0.7+default-dev | librust-sequoia-policy-config-0.6+default-dev");

    let expected = " librust-sequoia-policy-config-0.7+default-dev <!nocheck> | librust-sequoia-policy-config-0.6+default-dev <!nocheck>";

    assert_eq!(expected, result);
}

#[test]
fn test_deb_dep_add_nocheck_single_dep() {
    let result = deb_dep_add_nocheck("librust-serde-dev (>= 1.0)");
    assert_eq!("librust-serde-dev (>= 1.0) <!nocheck>", result);
}

#[test]
fn test_deb_dep_add_nocheck_no_version() {
    let result = deb_dep_add_nocheck("librust-serde-dev");
    assert_eq!("librust-serde-dev <!nocheck>", result);
}

#[test]
fn test_deb_dep_add_nocheck_three_alternatives() {
    let result = deb_dep_add_nocheck("a-dev | b-dev | c-dev");
    assert_eq!(
        "a-dev <!nocheck> | b-dev <!nocheck> | c-dev <!nocheck>",
        result
    );
}

#[test]
fn test_deb_dep_add_nocheck_versioned_alternatives() {
    let result = deb_dep_add_nocheck("librust-foo-1-dev (>= 1.2) | librust-foo-dev (>= 1.0)");
    assert_eq!(
        "librust-foo-1-dev (>= 1.2) <!nocheck> | librust-foo-dev (>= 1.0) <!nocheck>",
        result
    );
}

#[test]
fn test_v_ordering() {
    let v1 = V::new(&semver::Comparator::parse("1.0.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("2.0.0").unwrap()).unwrap();
    let v3 = V::new(&semver::Comparator::parse("1.1.0").unwrap()).unwrap();
    let v4 = V::new(&semver::Comparator::parse("1.0.1").unwrap()).unwrap();

    assert!(v1 < v2);
    assert!(v1 < v3);
    assert!(v1 < v4);
    assert!(v3 < v2);
    assert!(v4 < v3);
}

#[test]
fn test_v_inclast_major() {
    let v = V::new(&semver::Comparator::parse("5").unwrap()).unwrap();
    assert_eq!((6, 0, 0), v.inclast().mmp());
}

#[test]
fn test_v_inclast_minor() {
    let v = V::new(&semver::Comparator::parse("1.5").unwrap()).unwrap();
    assert_eq!((1, 6, 0), v.inclast().mmp());
}

#[test]
fn test_v_inclast_patch() {
    let v = V::new(&semver::Comparator::parse("1.5.9").unwrap()).unwrap();
    assert_eq!((1, 5, 10), v.inclast().mmp());
}

#[test]
fn test_v_display_major_only() {
    let v = V::new(&semver::Comparator::parse("3").unwrap()).unwrap();
    assert_eq!("3", v.to_string());
}

#[test]
fn test_v_display_major_minor() {
    let v = V::new(&semver::Comparator::parse("3.14").unwrap()).unwrap();
    assert_eq!("3.14", v.to_string());
}

#[test]
fn test_v_display_full() {
    let v = V::new(&semver::Comparator::parse("3.14.159").unwrap()).unwrap();
    assert_eq!("3.14.159", v.to_string());
}

#[test]
fn test_v_range_constrain_ge_keeps_highest() {
    let v1 = V::new(&semver::Comparator::parse("1.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("2.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_ge(v2.clone());

    assert_eq!(vr.ge.as_ref().unwrap(), &v2);
}

#[test]
fn test_v_range_constrain_lt_keeps_lowest() {
    let v1 = V::new(&semver::Comparator::parse("3.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("5.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_lt(v2);
    vr.constrain_lt(v1.clone());

    assert_eq!(vr.lt.as_ref().unwrap(), &v1);
}

#[test]
fn test_v_range_same_ge_and_lt_is_error() {
    let v = V::new(&semver::Comparator::parse("1.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v.clone());
    vr.constrain_lt(v);

    assert!(vr.to_deb_clause("base", "-dev").is_err());
}

#[test]
fn test_v_range_to_deb_clause_with_dev_suffix() {
    let v1 = V::new(&semver::Comparator::parse("1.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("2.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        vec!["base-1-dev"],
        vr.to_deb_clause("base", "-dev").unwrap()
    );
}

#[test]
fn test_v_equality_different_precision() {
    // 1.0 and 1.0.0 should be equal (both expand to (1,0,0))
    let v1 = V::new(&semver::Comparator::parse("1.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("1.0.0").unwrap()).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_v_zero_versions() {
    let v = V::new(&semver::Comparator::parse("0.0.0").unwrap()).unwrap();
    assert_eq!((0, 0, 0), v.mmp());
    assert_eq!((0, 0, 1), v.inclast().mmp());
}
