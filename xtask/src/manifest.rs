//! Pure text operations on `Cargo.toml`, scoped to the `[package]` table's `version` key.

use crate::error::XtaskError;
use crate::version::Version;

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

#[cfg(test)]
mod tests {
    use super::{package_version, with_package_version};
    use crate::error::XtaskError;
    use crate::version::Version;

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
}
