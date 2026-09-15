//! Changelog graduation and per version release notes extraction, Keep a Changelog format.

use crate::error::XtaskError;
use crate::version::Version;

/// The exact heading line a well formed `Unreleased` section must use.
const UNRELEASED_HEADING: &str = "## [Unreleased]";

/// The exact prefix of the `Unreleased` link reference definition.
const UNRELEASED_LINK_PREFIX: &str = "[Unreleased]: ";

/// Returns whether `line` is a link reference definition, `[label]: target`.
fn is_link_reference_line(line: &str) -> bool {
    line.starts_with('[') && line.contains("]:")
}

/// Returns whether `line` is the release heading for exactly `version`: it starts
/// with `## [version]` followed by either end of line or ` - `, so `0.1.1` never
/// matches `0.1.10` or `0.1.1-rc.1`.
fn heading_matches_version(line: &str, version: &str) -> bool {
    let prefix = format!("## [{version}]");
    line.strip_prefix(prefix.as_str())
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(" - "))
}

/// Finds the index of the exact `## [Unreleased]` heading line.
///
/// A heading that mentions "unreleased" in any other form is reported as
/// [`XtaskError::MalformedUnreleasedHeading`], which takes priority over
/// [`XtaskError::MissingUnreleasedSection`] when no exact heading exists.
fn find_unreleased_heading_index(lines: &[&str]) -> Result<usize, XtaskError> {
    let mut malformed_heading: Option<String> = None;
    for (index, line) in lines.iter().enumerate() {
        if *line == UNRELEASED_HEADING {
            return Ok(index);
        }
        if malformed_heading.is_none()
            && line.starts_with("## ")
            && line.to_lowercase().contains("unreleased")
        {
            malformed_heading = Some((*line).to_string());
        }
    }
    match malformed_heading {
        Some(line) => Err(XtaskError::MalformedUnreleasedHeading { line }),
        None => Err(XtaskError::MissingUnreleasedSection),
    }
}

/// Returns the index right after a section's body: the next `## ` heading, the
/// first link reference definition, or the end of the document.
fn find_section_end(lines: &[&str], start: usize) -> usize {
    lines[start..]
        .iter()
        .position(|line| line.starts_with("## ") || is_link_reference_line(line))
        .map_or(lines.len(), |offset| start + offset)
}

/// Trims blank lines (empty once trimmed) from both ends of `lines`.
fn trim_blank_lines<'lines>(lines: &'lines [&'lines str]) -> &'lines [&'lines str] {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    &lines[start..end]
}

/// Extracts the previous released version from the `Unreleased` compare link,
/// `[Unreleased]: <prefix>v<previous>...HEAD`, returning the URL prefix and the
/// previous version.
fn split_unreleased_link(link_line: &str) -> Option<(&str, &str)> {
    let target = link_line.strip_prefix(UNRELEASED_LINK_PREFIX)?;
    let compare_marker = "compare/v";
    let compare_start = target.find(compare_marker)?;
    let prefix = &target[..compare_start + compare_marker.len() - "v".len()];
    let after_marker = &target[compare_start + compare_marker.len()..];
    let previous_version = after_marker.strip_suffix("...HEAD")?;
    Some((prefix, previous_version))
}

/// Moves the `## [Unreleased]` body under a new `## [version] - date` heading,
/// updates the link reference definitions accordingly, and keeps an empty
/// `## [Unreleased]` heading at the top.
pub(crate) fn graduate(
    changelog: &str,
    version: &Version,
    date: &str,
) -> Result<String, XtaskError> {
    let lines: Vec<&str> = changelog.split('\n').collect();

    let heading_index = find_unreleased_heading_index(&lines)?;
    let section_end = find_section_end(&lines, heading_index + 1);
    let raw_body = &lines[heading_index + 1..section_end];
    let trimmed_body = trim_blank_lines(raw_body);
    // Subheadings such as `### Added` left without any entry are not content.
    if !trimmed_body.iter().any(|line| line.starts_with("- ")) {
        return Err(XtaskError::EmptyUnreleasedSection);
    }

    let version_string = version.to_string();
    if lines
        .iter()
        .any(|line| heading_matches_version(line, &version_string))
    {
        return Err(XtaskError::VersionAlreadyReleased {
            version: version_string,
        });
    }

    let link_index = lines[section_end..]
        .iter()
        .position(|line| line.starts_with(UNRELEASED_LINK_PREFIX))
        .map(|offset| section_end + offset)
        .ok_or(XtaskError::MissingUnreleasedLink)?;
    let (compare_prefix, previous_version) =
        split_unreleased_link(lines[link_index]).ok_or(XtaskError::MissingUnreleasedLink)?;

    let new_unreleased_link = format!("[Unreleased]: {compare_prefix}v{version_string}...HEAD");
    let new_version_link =
        format!("[{version_string}]: {compare_prefix}v{previous_version}...v{version_string}");

    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len() + 4);
    result_lines.extend(
        lines[..heading_index]
            .iter()
            .map(|line| (*line).to_string()),
    );
    result_lines.push(UNRELEASED_HEADING.to_string());
    result_lines.push(String::new());
    result_lines.push(format!("## [{version_string}] - {date}"));
    result_lines.push(String::new());
    result_lines.extend(trimmed_body.iter().map(|line| (*line).to_string()));
    result_lines.push(String::new());
    for (offset, line) in lines[section_end..].iter().enumerate() {
        if section_end + offset == link_index {
            result_lines.push(new_unreleased_link.clone());
            result_lines.push(new_version_link.clone());
        } else {
            result_lines.push((*line).to_string());
        }
    }

    Ok(result_lines.join("\n"))
}

/// Extracts the body of the `## [version]` section, trimmed of leading and trailing
/// blank lines, without its heading.
pub(crate) fn release_notes(changelog: &str, version: &str) -> Result<String, XtaskError> {
    let lines: Vec<&str> = changelog.split('\n').collect();

    let heading_index = lines
        .iter()
        .position(|line| heading_matches_version(line, version))
        .ok_or_else(|| XtaskError::SectionNotFound {
            version: version.to_string(),
        })?;

    let section_end = find_section_end(&lines, heading_index + 1);
    let body = &lines[heading_index + 1..section_end];
    let trimmed_body = trim_blank_lines(body);

    if trimmed_body.is_empty() {
        return Err(XtaskError::EmptySection {
            version: version.to_string(),
        });
    }

    Ok(trimmed_body.join("\n"))
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
    fn unreleased_section_with_only_empty_subheadings_is_reported() {
        let changelog = r"## [Unreleased]

### Added

### Fixed

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
