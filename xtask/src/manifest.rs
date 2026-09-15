//! Pure text operations on `Cargo.toml`, scoped to the `[package]` table's `version` key.

use crate::error::XtaskError;
use crate::version::Version;

/// Reads the `version` key of the `[package]` table.
pub(crate) fn package_version(_manifest: &str) -> Result<Version, XtaskError> {
    todo!()
}

/// Returns `manifest` with the `[package]` table's `version` key replaced by `version`,
/// leaving everything else byte for byte identical.
pub(crate) fn with_package_version(
    _manifest: &str,
    _version: &Version,
) -> Result<String, XtaskError> {
    todo!()
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
