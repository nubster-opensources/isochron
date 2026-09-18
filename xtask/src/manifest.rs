//! Pure text operations on `Cargo.toml`, scoped to the `[package]` table's `version` key
//! and to the `[package.metadata.isochron]` table's `next-release` declaration.

use crate::error::XtaskError;
use crate::version::{BumpLevel, Version};

/// The `[package]` table header, matched by exact line comparison after trimming.
const PACKAGE_TABLE_HEADING: &str = "[package]";

/// The `version` key prefix, matched at the start of a trimmed line.
const VERSION_KEY_PREFIX: &str = "version = \"";

/// Returns the `(start, end)` byte range of the quoted value of the `version` key
/// in `line`, if `line` (once its leading whitespace is trimmed) starts with the
/// `version` key. This rejects lookalike keys such as `rust-version = "..."`.
fn version_value_range(line: &str) -> Option<(usize, usize)> {
    let leading_whitespace_length = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix(VERSION_KEY_PREFIX)?;
    let value_start = leading_whitespace_length + VERSION_KEY_PREFIX.len();
    let relative_value_end = rest.find('"')?;
    Some((value_start, value_start + relative_value_end))
}

/// Reads the `version` key of the `[package]` table.
pub(crate) fn package_version(manifest: &str) -> Result<Version, XtaskError> {
    let mut in_package_table = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package_table = trimmed == PACKAGE_TABLE_HEADING;
            continue;
        }
        if !in_package_table {
            continue;
        }
        let Some((value_start, value_end)) = version_value_range(line) else {
            continue;
        };
        return line[value_start..value_end].parse::<Version>();
    }
    Err(XtaskError::ManifestVersionNotFound)
}

/// Returns `manifest` with the `[package]` table's `version` key replaced by `version`,
/// leaving everything else byte for byte identical.
pub(crate) fn with_package_version(
    manifest: &str,
    version: &Version,
) -> Result<String, XtaskError> {
    let mut in_package_table = false;
    let mut replaced = false;
    let mut result = String::with_capacity(manifest.len() + 4);

    for raw_line in manifest.split_inclusive('\n') {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let trimmed = line.trim();

        if !replaced && trimmed.starts_with('[') {
            in_package_table = trimmed == PACKAGE_TABLE_HEADING;
            result.push_str(raw_line);
            continue;
        }

        if replaced || !in_package_table {
            result.push_str(raw_line);
            continue;
        }

        let Some((value_start, value_end)) = version_value_range(line) else {
            result.push_str(raw_line);
            continue;
        };

        result.push_str(&line[..value_start]);
        result.push_str(&version.to_string());
        result.push_str(&line[value_end..]);
        if raw_line.ends_with('\n') {
            result.push('\n');
        }
        replaced = true;
    }

    if replaced {
        Ok(result)
    } else {
        Err(XtaskError::ManifestVersionNotFound)
    }
}

/// The `[package.metadata.isochron]` table header, matched by exact line comparison
/// after trimming.
const ISOCHRON_METADATA_TABLE_HEADING: &str = "[package.metadata.isochron]";

/// The `next-release` key prefix, matched at the start of a trimmed line.
const NEXT_RELEASE_KEY_PREFIX: &str = "next-release = \"";

/// Returns the `(start, end)` byte range of the quoted value of the `next-release` key in
/// `line`, if `line` (once its leading whitespace is trimmed) starts with that key.
fn next_release_value_range_in_line(line: &str) -> Option<(usize, usize)> {
    let leading_whitespace_length = line.len() - line.trim_start().len();
    let rest = line.trim_start().strip_prefix(NEXT_RELEASE_KEY_PREFIX)?;
    let value_start = leading_whitespace_length + NEXT_RELEASE_KEY_PREFIX.len();
    let relative_value_end = rest.find('"')?;
    Some((value_start, value_start + relative_value_end))
}

/// Returns the byte offset of the line that declares `next-release` inside the
/// `[package.metadata.isochron]` table, and the range of its quoted value within that line.
///
/// The table scope is what makes the lookup safe: `next-release` under any other table,
/// `[package]` or another `metadata` table, is not this declaration and is ignored.
fn next_release_value_range(manifest: &str) -> Option<(usize, usize, usize)> {
    let mut in_isochron_table = false;
    let mut line_start = 0;

    for raw_line in manifest.split_inclusive('\n') {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_isochron_table = trimmed == ISOCHRON_METADATA_TABLE_HEADING;
        } else if in_isochron_table
            && let Some((value_start, value_end)) = next_release_value_range_in_line(line)
        {
            return Some((line_start, value_start, value_end));
        }

        line_start += raw_line.len();
    }

    None
}

/// Reads the `next-release` declaration of the `[package.metadata.isochron]` table.
///
/// The declaration is mandatory: a missing key is an error, never an implicit `patch`, so
/// that a misspelt table cannot silently change which release type the API check applies.
pub(crate) fn declared_next_release(manifest: &str) -> Result<BumpLevel, XtaskError> {
    let (value_start, value_end) = next_release_value_range(manifest)
        .map(|(line_start, start, end)| (line_start + start, line_start + end))
        .ok_or(XtaskError::NextReleaseNotFound)?;

    match &manifest[value_start..value_end] {
        "patch" => Ok(BumpLevel::Patch),
        "minor" => Ok(BumpLevel::Minor),
        "major" => Ok(BumpLevel::Major),
        other => Err(XtaskError::InvalidNextRelease {
            value: other.to_string(),
        }),
    }
}

/// Returns `manifest` with the `next-release` declaration replaced by `level`, leaving
/// everything else byte for byte identical.
pub(crate) fn with_declared_next_release(
    manifest: &str,
    level: BumpLevel,
) -> Result<String, XtaskError> {
    let (line_start, value_start, value_end) =
        next_release_value_range(manifest).ok_or(XtaskError::NextReleaseNotFound)?;

    let mut result = String::with_capacity(manifest.len() + 4);
    result.push_str(&manifest[..line_start + value_start]);
    result.push_str(level.as_str());
    result.push_str(&manifest[line_start + value_end..]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        declared_next_release, package_version, with_declared_next_release, with_package_version,
    };
    use crate::error::XtaskError;
    use crate::version::{BumpLevel, Version};

    const DECLARED_MINOR_MANIFEST: &str = "[package]
name = \"isochron\"
version = \"0.1.2\"

[package.metadata.docs.rs]
all-features = true

[package.metadata.isochron]
next-release = \"minor\"

[dependencies]
thiserror = \"2\"
";

    const FIXTURE_MANIFEST: &str = "[package]\n\
name = \"isochron\"\n\
version = \"0.1.1\"\n\
edition = \"2024\"\n\
rust-version = \"1.89\"\n\
\n\
[dependencies]\n\
time = { version = \"0.3\", features = [\"macros\"] }\n\
\n\
[workspace]\n\
members = [\"xtask\"]\n";

    #[test]
    fn reads_the_package_version() {
        assert_eq!(
            package_version(FIXTURE_MANIFEST).unwrap(),
            Version::new(0, 1, 1)
        );
    }

    #[test]
    fn reports_missing_version_key() {
        let manifest = "[package]\nname = \"isochron\"\n";
        assert!(matches!(
            package_version(manifest),
            Err(XtaskError::ManifestVersionNotFound)
        ));
    }

    #[test]
    fn reports_unparsable_version_value() {
        let manifest = "[package]\nname = \"isochron\"\nversion = \"not-a-version\"\n";
        assert!(matches!(
            package_version(manifest),
            Err(XtaskError::InvalidVersion { .. })
        ));
    }

    #[test]
    fn replaces_only_the_package_version() {
        let updated = with_package_version(FIXTURE_MANIFEST, &Version::new(0, 2, 0)).unwrap();
        let expected = FIXTURE_MANIFEST.replace("version = \"0.1.1\"", "version = \"0.2.0\"");
        assert_eq!(updated, expected);
        assert_eq!(package_version(&updated).unwrap(), Version::new(0, 2, 0));
    }

    #[test]
    fn real_manifest_version_round_trips() {
        let real_manifest = include_str!("../../Cargo.toml");
        let current = package_version(real_manifest).unwrap();
        let rewritten = with_package_version(real_manifest, &current).unwrap();
        assert_eq!(rewritten, real_manifest);
    }

    #[test]
    fn reads_each_declared_next_release_level() {
        for (value, level) in [
            ("patch", BumpLevel::Patch),
            ("minor", BumpLevel::Minor),
            ("major", BumpLevel::Major),
        ] {
            let manifest = DECLARED_MINOR_MANIFEST.replace(
                "next-release = \"minor\"",
                &format!("next-release = \"{value}\""),
            );
            assert_eq!(declared_next_release(&manifest).unwrap(), level);
        }
    }

    #[test]
    fn reports_a_manifest_without_the_isochron_metadata_table() {
        let manifest = "[package]
name = \"isochron\"
version = \"0.1.2\"
";
        assert!(matches!(
            declared_next_release(manifest),
            Err(XtaskError::NextReleaseNotFound)
        ));
    }

    #[test]
    fn ignores_a_next_release_key_outside_the_isochron_metadata_table() {
        let manifest = "[package]
name = \"isochron\"
next-release = \"minor\"

[package.metadata.docs.rs]
next-release = \"minor\"
";
        assert!(matches!(
            declared_next_release(manifest),
            Err(XtaskError::NextReleaseNotFound)
        ));
    }

    #[test]
    fn reports_an_unknown_next_release_value_verbatim() {
        let manifest =
            DECLARED_MINOR_MANIFEST.replace("next-release = \"minor\"", "next-release = \"Minor\"");
        assert!(matches!(
            declared_next_release(&manifest),
            Err(XtaskError::InvalidNextRelease { value }) if value == "Minor"
        ));
    }

    #[test]
    fn replaces_only_the_next_release_declaration() {
        let updated =
            with_declared_next_release(DECLARED_MINOR_MANIFEST, BumpLevel::Patch).unwrap();
        let expected =
            DECLARED_MINOR_MANIFEST.replace("next-release = \"minor\"", "next-release = \"patch\"");
        assert_eq!(updated, expected);
        assert_eq!(declared_next_release(&updated).unwrap(), BumpLevel::Patch);
    }

    #[test]
    fn reports_a_missing_declaration_when_rewriting() {
        let manifest = "[package]
name = \"isochron\"
version = \"0.1.2\"
";
        assert!(matches!(
            with_declared_next_release(manifest, BumpLevel::Patch),
            Err(XtaskError::NextReleaseNotFound)
        ));
    }

    #[test]
    fn real_manifest_next_release_round_trips() {
        let real_manifest = include_str!("../../Cargo.toml");
        let declared = declared_next_release(real_manifest).unwrap();
        let rewritten = with_declared_next_release(real_manifest, declared).unwrap();
        assert_eq!(rewritten, real_manifest);
    }
}
