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
fn test_v_range_to_deb_or_clause_empty() {
    let vr = VRange::new();

    assert_eq!(
        "base+feature",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_ge_only() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);

    assert_eq!(
        "base+feature (>= 0.9-~~)",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_lt_only() {
    let v1 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_lt(v1);

    assert_eq!(
        "base+feature (<< 0.10-~~)",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_or_clause_minor_one_apart() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.10").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        "base-0.9+feature",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_or_clause_minor_two_apart() {
    let v1 = V::new(&semver::Comparator::parse("0.9").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("0.11").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        "base-0.10+feature | base-0.9+feature",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_or_clause_major_one_apart() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("10.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        "base-9+feature",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_or_clause_major_two_apart() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("11.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v1);
    vr.constrain_lt(v2);

    assert_eq!(
        "base-10+feature | base-9+feature",
        vr.to_deb_or_clause("base", "+feature").unwrap()
    )
}

#[test]
fn test_v_range_to_deb_or_clause_bad_range() {
    let v1 = V::new(&semver::Comparator::parse("9.0").unwrap()).unwrap();
    let v2 = V::new(&semver::Comparator::parse("11.0").unwrap()).unwrap();

    let mut vr = VRange::new();
    vr.constrain_ge(v2);
    vr.constrain_lt(v1);

    let err = vr.to_deb_or_clause("base", "+feature");
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
