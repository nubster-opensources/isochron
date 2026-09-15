//! Entry point for the isochron repository automation tasks: release preparation and
//! release notes extraction, replacing the former release shell script and tooling.

mod changelog;
mod civil_date;
mod error;
mod manifest;
mod release_prep;
mod version;

use crate::error::XtaskError;
use crate::version::VersionRequest;

/// A fully parsed command line invocation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Invocation {
    /// Prepare a release: bump the manifest, graduate the changelog, verify, and open a pull request.
    ReleasePrep {
        /// The requested next version.
        request: VersionRequest,
        /// Whether to skip the network side effects, the push and the pull request.
        is_dry_run: bool,
    },
    /// Print the release notes body for an already released version.
    ReleaseNotes {
        /// The version whose release notes body to print.
        version: String,
    },
}

/// Parses the process command line arguments, excluding the program name, into an [`Invocation`].
pub(crate) fn parse_invocation(_arguments: &[String]) -> Result<Invocation, XtaskError> {
    todo!()
}

fn main() -> std::process::ExitCode {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{Invocation, parse_invocation};
    use crate::error::XtaskError;
    use crate::version::{BumpLevel, Version, VersionRequest};

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_a_bump_level_release_prep() {
        let invocation = parse_invocation(&arguments(&["release-prep", "patch"])).unwrap();
        assert_eq!(
            invocation,
            Invocation::ReleasePrep {
                request: VersionRequest::Level(BumpLevel::Patch),
                is_dry_run: false
            }
        );
    }

    #[test]
    fn parses_an_exact_version_dry_run_release_prep() {
        let invocation =
            parse_invocation(&arguments(&["release-prep", "0.2.0", "--dry-run"])).unwrap();
        assert_eq!(
            invocation,
            Invocation::ReleasePrep {
                request: VersionRequest::Exact(Version::new(0, 2, 0)),
                is_dry_run: true,
            }
        );
    }

    #[test]
    fn parses_release_notes() {
        let invocation = parse_invocation(&arguments(&["release-notes", "0.1.1"])).unwrap();
        assert_eq!(
            invocation,
            Invocation::ReleaseNotes {
                version: "0.1.1".to_string()
            }
        );
    }

    #[test]
    fn rejects_no_arguments() {
        assert!(matches!(
            parse_invocation(&arguments(&[])),
            Err(XtaskError::Usage(_))
        ));
    }

    #[test]
    fn rejects_an_unknown_command() {
        assert!(matches!(
            parse_invocation(&arguments(&["publish"])),
            Err(XtaskError::Usage(_))
        ));
    }

    #[test]
    fn rejects_release_prep_without_a_version() {
        assert!(matches!(
            parse_invocation(&arguments(&["release-prep"])),
            Err(XtaskError::Usage(_))
        ));
    }

    #[test]
    fn rejects_release_notes_without_a_version() {
        assert!(matches!(
            parse_invocation(&arguments(&["release-notes"])),
            Err(XtaskError::Usage(_))
        ));
    }

    #[test]
    fn rejects_release_prep_with_an_unknown_flag() {
        assert!(matches!(
            parse_invocation(&arguments(&["release-prep", "patch", "--force"])),
            Err(XtaskError::Usage(_))
        ));
    }
}
