//! Changelog graduation and per version release notes extraction, Keep a Changelog format.

use crate::error::XtaskError;
use crate::version::Version;

/// Moves the `## [Unreleased]` body under a new `## [version] - date` heading,
/// updates the link reference definitions accordingly, and keeps an empty
/// `## [Unreleased]` heading at the top.
pub(crate) fn graduate(
    _changelog: &str,
    _version: &Version,
    _date: &str,
) -> Result<String, XtaskError> {
    todo!()
}

/// Extracts the body of the `## [version]` section, trimmed of leading and trailing
/// blank lines, without its heading.
pub(crate) fn release_notes(_changelog: &str, _version: &str) -> Result<String, XtaskError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{graduate, release_notes};
    use crate::error::XtaskError;
    use crate::version::Version;

    const FIXTURE_CHANGELOG: &str = r"## [Unreleased]

### Added

- Add new thing. (#1)

### Fixed

- Fix a bug. (#2)

## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/nubster-opensources/isochron/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
";

    #[test]
    fn graduates_the_unreleased_section_exactly() {
        let expected = r"## [Unreleased]

## [0.1.2] - 2026-09-15

### Added

- Add new thing. (#1)

### Fixed

- Fix a bug. (#2)

## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/nubster-opensources/isochron/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
";

        let graduated = graduate(FIXTURE_CHANGELOG, &Version::new(0, 1, 2), "2026-09-15").unwrap();
        assert_eq!(graduated, expected);
    }

    #[test]
    fn graduated_heading_carries_the_date_exactly_once() {
        let graduated = graduate(FIXTURE_CHANGELOG, &Version::new(0, 1, 2), "2026-09-15").unwrap();
        let occurrences = graduated.matches("## [0.1.2] - 2026-09-15").count();
        assert_eq!(occurrences, 1);
        assert!(!graduated.contains("## [0.1.2]\n"));
        assert!(!graduated.to_lowercase().contains("(unreleased)"));
    }

    #[test]
    fn missing_unreleased_section_is_reported() {
        let changelog = r"## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(error, XtaskError::MissingUnreleasedSection));
    }

    #[test]
    fn malformed_unreleased_heading_without_brackets_is_reported() {
        let changelog = r"## Unreleased

- Something pending.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(
            error,
            XtaskError::MalformedUnreleasedHeading { line } if line == "## Unreleased"
        ));
    }

    #[test]
    fn malformed_unreleased_heading_takes_priority_over_missing_section() {
        let changelog = r"## [unreleased]

- Something pending.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(
            error,
            XtaskError::MalformedUnreleasedHeading { .. }
        ));
    }

    #[test]
    fn malformed_unreleased_heading_embedded_in_version_heading_is_reported() {
        let changelog = r"## 0.1.2 (unreleased)

- Something pending.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(
            error,
            XtaskError::MalformedUnreleasedHeading { .. }
        ));
    }

    #[test]
    fn empty_unreleased_section_is_reported() {
        let changelog = r"## [Unreleased]

## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(error, XtaskError::EmptyUnreleasedSection));
    }

    #[test]
    fn version_already_released_is_reported() {
        let changelog = r"## [Unreleased]

- Pending change.

## [0.1.2] - 2026-08-01

### Added

- Already released.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.2
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(
            error,
            XtaskError::VersionAlreadyReleased { version } if version == "0.1.2"
        ));
    }

    #[test]
    fn missing_unreleased_link_is_reported() {
        let changelog = r"## [Unreleased]

- Pending change.

## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let error = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap_err();
        assert!(matches!(error, XtaskError::MissingUnreleasedLink));
    }

    #[test]
    fn extracts_body_between_adjacent_headings() {
        let changelog = r"## [Unreleased]

- Nothing yet.

## [0.2.0] - 2026-08-01

### Added

- Feature two.

## [0.1.0] - 2026-07-01

### Added

- Feature one.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nubster-opensources/isochron/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
";
        let notes = release_notes(changelog, "0.2.0").unwrap();
        assert_eq!(notes, "### Added\n\n- Feature two.");
    }

    #[test]
    fn does_not_match_versions_sharing_a_prefix() {
        let changelog = r"## [0.1.10] - 2026-09-01

### Added

- Ten body.

## [0.1.1-rc.1] - 2026-08-15

### Added

- Release candidate body.

## [0.1.1] - 2026-06-18

### Changed

- Stricter field parsing text.

[0.1.10]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.10
[0.1.1-rc.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1-rc.1
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let notes = release_notes(changelog, "0.1.1").unwrap();
        assert_eq!(notes, "### Changed\n\n- Stricter field parsing text.");
    }

    #[test]
    fn oldest_section_stops_before_link_definitions() {
        let changelog = r"## [0.1.0] - 2026-06-15

### Added

- Initial release.

[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
";
        let notes = release_notes(changelog, "0.1.0").unwrap();
        assert_eq!(notes, "### Added\n\n- Initial release.");
    }

    #[test]
    fn missing_version_is_reported() {
        let changelog = r"## [0.1.0] - 2026-06-15

### Added

- Initial release.

[0.1.0]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.0
";
        let error = release_notes(changelog, "0.9.0").unwrap_err();
        assert!(matches!(error, XtaskError::SectionNotFound { version } if version == "0.9.0"));
    }

    #[test]
    fn empty_section_is_reported() {
        let changelog = r"## [0.1.2] - 2026-09-15

## [0.1.1] - 2026-06-18

### Changed

- Something changed.

[0.1.2]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.2
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let error = release_notes(changelog, "0.1.2").unwrap_err();
        assert!(matches!(error, XtaskError::EmptySection { version } if version == "0.1.2"));
    }

    #[test]
    fn real_changelog_release_notes_for_0_1_1_match_the_repository_file() {
        let real_changelog = include_str!("../../CHANGELOG.md");
        let expected = "### Changed\n\n\
- Stricter field parsing: a leading `+` sign (`+5`) or zero-padded numbers (`007`, `00`) are now rejected as non-canonical, in both values and `/N` step tokens, matching standard Vixie cron. The single digit `0` stays valid.\n\
- The `Upcoming` iterator type is now `#[must_use]`, so binding it without consuming the iterator is linted.";
        assert_eq!(release_notes(real_changelog, "0.1.1").unwrap(), expected);
    }

    #[test]
    fn release_notes_round_trips_the_graduated_unreleased_body() {
        let changelog = r"## [Unreleased]

### Added

- New feature bullet.

## [0.1.1] - 2026-06-18

### Changed

- Old change bullet.

[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1
";
        let graduated = graduate(changelog, &Version::new(0, 1, 2), "2026-09-15").unwrap();
        let notes = release_notes(&graduated, "0.1.2").unwrap();
        assert_eq!(notes, "### Added\n\n- New feature bullet.");
    }
}
