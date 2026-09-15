//! Strict `SemVer` core version type and version bump requests.

use crate::error::XtaskError;

/// A strict `SemVer` core version: `major.minor.patch`, no prerelease or build metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// Builds a version from its three numeric components.
    pub(crate) const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Parses one dot separated component of a version: digits only, and no
/// leading zero unless the component is exactly `0`.
fn parse_strict_component(component: &str) -> Option<u64> {
    if component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if component != "0" && component.starts_with('0') {
        return None;
    }
    component.parse::<u64>().ok()
}

impl std::str::FromStr for Version {
    type Err = XtaskError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let invalid = || XtaskError::InvalidVersion {
            input: input.to_string(),
        };

        let mut components = input.split('.');
        let major = components.next().ok_or_else(invalid)?;
        let minor = components.next().ok_or_else(invalid)?;
        let patch = components.next().ok_or_else(invalid)?;
        if components.next().is_some() {
            return Err(invalid());
        }

        let major = parse_strict_component(major).ok_or_else(invalid)?;
        let minor = parse_strict_component(minor).ok_or_else(invalid)?;
        let patch = parse_strict_component(patch).ok_or_else(invalid)?;

        Ok(Self::new(major, minor, patch))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A semantic version component to increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BumpLevel {
    /// Increment the patch component.
    Patch,
    /// Increment the minor component and reset the patch component.
    Minor,
    /// Increment the major component and reset the minor and patch components.
    Major,
}

/// A requested next version: either a relative bump or an exact target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionRequest {
    /// Bump the current version by one level.
    Level(BumpLevel),
    /// Use this exact version, provided it is strictly greater than the current one.
    Exact(Version),
}

impl std::str::FromStr for VersionRequest {
    type Err = XtaskError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "patch" => Ok(Self::Level(BumpLevel::Patch)),
            "minor" => Ok(Self::Level(BumpLevel::Minor)),
            "major" => Ok(Self::Level(BumpLevel::Major)),
            _ => input.parse::<Version>().map(Self::Exact),
        }
    }
}

impl VersionRequest {
    /// Refuses a target that is not strictly greater than `current`.
    pub(crate) fn resolve(&self, current: &Version) -> Result<Version, XtaskError> {
        let target = match self {
            Self::Level(BumpLevel::Patch) => {
                Version::new(current.major, current.minor, current.patch + 1)
            }
            Self::Level(BumpLevel::Minor) => Version::new(current.major, current.minor + 1, 0),
            Self::Level(BumpLevel::Major) => Version::new(current.major + 1, 0, 0),
            Self::Exact(version) => *version,
        };

        if target <= *current {
            return Err(XtaskError::VersionNotGreater {
                current: current.to_string(),
                target: target.to_string(),
            });
        }

        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::{BumpLevel, Version, VersionRequest};
    use crate::error::XtaskError;

    #[test]
    fn parses_minimal_versions() {
        assert_eq!("0.1.2".parse::<Version>().unwrap(), Version::new(0, 1, 2));
        assert_eq!("0.0.0".parse::<Version>().unwrap(), Version::new(0, 0, 0));
    }

    #[test]
    fn rejects_leading_zero_in_major() {
        let error = "07.1.2".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { input } if input == "07.1.2"));
    }

    #[test]
    fn rejects_leading_zero_in_minor() {
        let error = "0.01.2".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_missing_patch_component() {
        let error = "1.2".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_extra_component() {
        let error = "1.2.3.4".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_prerelease_suffix() {
        let error = "1.2.3-rc.1".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_leading_space() {
        let error = " 1.2.3".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_v_prefix() {
        let error = "v1.2.3".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn rejects_empty_input() {
        let error = "".parse::<Version>().unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    #[test]
    fn display_round_trips_through_parse() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        assert_eq!(version.to_string().parse::<Version>().unwrap(), version);
    }

    #[test]
    fn resolves_patch_bump() {
        let current = Version::new(0, 1, 1);
        let target = VersionRequest::Level(BumpLevel::Patch)
            .resolve(&current)
            .unwrap();
        assert_eq!(target, Version::new(0, 1, 2));
    }

    #[test]
    fn resolves_minor_bump() {
        let current = Version::new(0, 1, 1);
        let target = VersionRequest::Level(BumpLevel::Minor)
            .resolve(&current)
            .unwrap();
        assert_eq!(target, Version::new(0, 2, 0));
    }

    #[test]
    fn resolves_major_bump() {
        let current = Version::new(0, 1, 1);
        let target = VersionRequest::Level(BumpLevel::Major)
            .resolve(&current)
            .unwrap();
        assert_eq!(target, Version::new(1, 0, 0));
    }

    #[test]
    fn minor_bump_resets_patch() {
        let current = Version::new(1, 2, 3);
        let target = VersionRequest::Level(BumpLevel::Minor)
            .resolve(&current)
            .unwrap();
        assert_eq!(target, Version::new(1, 3, 0));
    }

    #[test]
    fn resolves_exact_greater_version() {
        let current = Version::new(0, 1, 1);
        let target = VersionRequest::Exact(Version::new(0, 1, 2))
            .resolve(&current)
            .unwrap();
        assert_eq!(target, Version::new(0, 1, 2));
    }

    #[test]
    fn rejects_exact_equal_version() {
        let current = Version::new(0, 1, 1);
        let error = VersionRequest::Exact(Version::new(0, 1, 1))
            .resolve(&current)
            .unwrap_err();
        assert!(matches!(error, XtaskError::VersionNotGreater { .. }));
    }

    #[test]
    fn rejects_exact_lesser_version() {
        let current = Version::new(0, 1, 1);
        let error = VersionRequest::Exact(Version::new(0, 1, 0))
            .resolve(&current)
            .unwrap_err();
        assert!(matches!(error, XtaskError::VersionNotGreater { .. }));
    }

    #[test]
    fn parses_bump_level_keywords_and_exact_versions() {
        assert!(matches!(
            "patch".parse::<VersionRequest>().unwrap(),
            VersionRequest::Level(BumpLevel::Patch)
        ));
        assert!(matches!(
            "minor".parse::<VersionRequest>().unwrap(),
            VersionRequest::Level(BumpLevel::Minor)
        ));
        assert!(matches!(
            "major".parse::<VersionRequest>().unwrap(),
            VersionRequest::Level(BumpLevel::Major)
        ));
        assert!(matches!(
            "0.2.0".parse::<VersionRequest>().unwrap(),
            VersionRequest::Exact(version) if version == Version::new(0, 2, 0)
        ));
    }

    #[test]
    fn rejects_unknown_version_request_keywords() {
        assert!(matches!(
            "Patch".parse::<VersionRequest>(),
            Err(XtaskError::InvalidVersion { .. })
        ));
        assert!(matches!(
            "pre".parse::<VersionRequest>(),
            Err(XtaskError::InvalidVersion { .. })
        ));
        assert!(matches!(
            "".parse::<VersionRequest>(),
            Err(XtaskError::InvalidVersion { .. })
        ));
    }
}
