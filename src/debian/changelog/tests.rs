use crate::debian::changelog::{ChangelogEntry, ChangelogIterator};
use chrono::{FixedOffset, TimeZone};
use std::str::FromStr;

/// changelog for rsa selected because 0.9.2-2 appears twice
const CHANGELOG_RSA: &str = r"rust-rsa (0.9.7-1) unstable; urgency=medium

  * Team upload.
  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5
  * Drop unneeded dependency adjustments

 -- Daniel Kahn Gillmor <dkg@fifthhorseman.net>  Thu, 05 Dec 2024 11:08:36 -0500

rust-rsa (0.9.6-1) unstable; urgency=medium

  * Package rsa 0.9.6 from crates.io using debcargo 2.7.0. Closes: ##1073127

 -- Jelmer Vernooĳ <jelmer@debian.org>  Wed, 25 Sep 2024 14:06:55 +0100

rust-rsa (0.9.2-2) unstable; urgency=medium

  * Add patch pkcs8: fix compatibility with pkcs8.

 -- Jelmer Vernooĳ <jelmer@debian.org>  Tue, 19 Sep 2023 21:27:07 +0100

rust-rsa (0.9.2-2) unstable; urgency=medium

  * Add patch pkcs8: fix compatibility with pkcs8.

 -- Jelmer Vernooĳ <jelmer@debian.org>  Tue, 19 Sep 2023 21:27:07 +0100

rust-rsa (0.9.2-1) unstable; urgency=medium

  * Package rsa 0.9.2 from crates.io using debcargo 2.6.0

 -- Jelmer Vernooĳ <jelmer@debian.org>  Mon, 18 Sep 2023 23:08:13 +0100

";

#[test]
fn test_changelog_iterator() {
    let mut iter = ChangelogIterator::from(CHANGELOG_RSA);

    let section = r"rust-rsa (0.9.7-1) unstable; urgency=medium

  * Team upload.
  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5
  * Drop unneeded dependency adjustments

 -- Daniel Kahn Gillmor <dkg@fifthhorseman.net>  Thu, 05 Dec 2024 11:08:36 -0500

";
    assert_eq!(section, iter.next().unwrap());
    let section = r"rust-rsa (0.9.6-1) unstable; urgency=medium

  * Package rsa 0.9.6 from crates.io using debcargo 2.7.0. Closes: ##1073127

 -- Jelmer Vernooĳ <jelmer@debian.org>  Wed, 25 Sep 2024 14:06:55 +0100

";
    assert_eq!(section, iter.next().unwrap());
    let section = r"rust-rsa (0.9.2-2) unstable; urgency=medium

  * Add patch pkcs8: fix compatibility with pkcs8.

 -- Jelmer Vernooĳ <jelmer@debian.org>  Tue, 19 Sep 2023 21:27:07 +0100

";
    assert_eq!(section, iter.next().unwrap());
    let section = r"rust-rsa (0.9.2-2) unstable; urgency=medium

  * Add patch pkcs8: fix compatibility with pkcs8.

 -- Jelmer Vernooĳ <jelmer@debian.org>  Tue, 19 Sep 2023 21:27:07 +0100

";
    assert_eq!(section, iter.next().unwrap());
    let section = r"rust-rsa (0.9.2-1) unstable; urgency=medium

  * Package rsa 0.9.2 from crates.io using debcargo 2.6.0

 -- Jelmer Vernooĳ <jelmer@debian.org>  Mon, 18 Sep 2023 23:08:13 +0100

";
    assert_eq!(section, iter.next().unwrap());

    // assert that the iterator ends
    assert_eq!(None, iter.next());
}

#[test]
fn test_from_string() {
    let mut iter = ChangelogIterator::from(CHANGELOG_RSA);

    let instance = ChangelogEntry::from_str(iter.next().unwrap()).unwrap();

    assert_eq!("rust-rsa", instance.source);
    assert_eq!(
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        instance.date
    );
    assert_eq!(
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned()
        ],
        instance.items
    );
    assert_eq!("0.9.7-1", instance.version);
    assert_eq!(
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>",
        instance.maintainer
    );
    assert_eq!(" unstable", instance.distribution);
    assert_eq!(" urgency=medium", instance.options);
}

#[test]
fn test_changelog_entry_display() {
    let instance = ChangelogEntry::new(
        "rust-rsa".to_owned(),
        "0.9.7-1".to_owned(),
        "unstable".to_owned(),
        "urgency=medium".to_owned(),
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>".to_owned(),
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned(),
        ],
    );

    let expected = r"rust-rsa (0.9.7-1) unstable; urgency=medium

  * Team upload.
  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5
  * Drop unneeded dependency adjustments

 -- Daniel Kahn Gillmor <dkg@fifthhorseman.net>  Thu, 5 Dec 2024 11:08:36 -0500
";

    assert_eq!(expected, instance.to_string());
}

#[test]
fn test_changelog_entry_maintainer_name() {
    let instance = ChangelogEntry::new(
        "rust-rsa".to_owned(),
        "0.9.7-1".to_owned(),
        "unstable".to_owned(),
        "urgency=medium".to_owned(),
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>".to_owned(),
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned(),
        ],
    );

    assert_eq!("Daniel Kahn Gillmor", instance.maintainer_name());
}

#[test]
fn test_changelog_entry_version_parts() {
    let instance = ChangelogEntry::new(
        "rust-rsa".to_owned(),
        "0.9.7-1".to_owned(),
        "unstable".to_owned(),
        "urgency=medium".to_owned(),
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>".to_owned(),
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned(),
        ],
    );

    assert_eq!(
        ("0.9.7".to_owned(), "1".to_owned()),
        instance.version_parts()
    );
}

#[test]
fn test_changelog_entry_deb_version_suffix() {
    let instance = ChangelogEntry::new(
        "rust-rsa".to_owned(),
        "0.9.7-1".to_owned(),
        "unstable".to_owned(),
        "urgency=medium".to_owned(),
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>".to_owned(),
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned(),
        ],
    );

    assert_eq!("1", instance.deb_version_suffix());
}

#[test]
fn test_changelog_entry_deb_version_suffix_bump() {
    let instance = ChangelogEntry::new(
        "rust-rsa".to_owned(),
        "0.9.7-1".to_owned(),
        "unstable".to_owned(),
        "urgency=medium".to_owned(),
        "Daniel Kahn Gillmor <dkg@fifthhorseman.net>".to_owned(),
        FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2024, 12, 05, 11, 08, 36)
            .unwrap(),
        vec![
            "  * Team upload.".to_owned(),
            "  * Package rsa 0.9.7 from crates.io using debcargo 2.7.5".to_owned(),
            "  * Drop unneeded dependency adjustments".to_owned(),
        ],
    );

    assert_eq!("2", instance.deb_version_suffix_bump());
}
