//! Entry point for the isochron repository automation tasks: release preparation and
//! release notes extraction, replacing the former release shell script and tooling.

mod changelog;
mod civil_date;
mod command_runner;
mod error;
mod manifest;
mod release_prep;
mod release_verify;
mod version;

use crate::error::XtaskError;
use crate::version::VersionRequest;

/// A fully parsed command line invocation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Invocation {
    /// Prepare a release: bump the manifest, graduate the changelog, verify, and open a pull request.
    PrepareRelease {
        /// The requested next version.
        request: VersionRequest,
        /// Whether to skip the network side effects, the push and the pull request.
        is_dry_run: bool,
    },
    /// Print the release notes body for an already released version.
    PrintReleaseNotes {
        /// The version whose release notes body to print.
        version: String,
    },
    /// Check that a pushed release tag may be published, and print its version.
    VerifyReleaseTag {
        /// The tag as pushed, `vX.Y.Z`.
        tag: String,
    },
}

/// Usage text listing every subcommand, shown when the command line cannot be interpreted.
const USAGE: &str = "Usage:\n  \
cargo xtask release-prep <patch|minor|major|x.y.z> [--dry-run]\n  \
cargo xtask release-notes <version>\n  \
cargo xtask release-verify <vX.Y.Z>";

/// Parses the process command line arguments, excluding the program name, into an [`Invocation`].
pub(crate) fn parse_invocation(arguments: &[String]) -> Result<Invocation, XtaskError> {
    let usage = || XtaskError::Usage(USAGE.to_string());

    let mut arguments = arguments.iter();
    let command = arguments.next().ok_or_else(usage)?;

    match command.as_str() {
        "release-prep" => {
            let version_argument = arguments.next().ok_or_else(usage)?;
            let request: VersionRequest = version_argument.parse()?;

            let mut is_dry_run = false;
            for extra in arguments {
                if extra.as_str() == "--dry-run" {
                    is_dry_run = true;
                } else {
                    return Err(usage());
                }
            }

            Ok(Invocation::PrepareRelease {
                request,
                is_dry_run,
            })
        }
        "release-notes" => {
            let version = arguments.next().ok_or_else(usage)?.clone();
            if arguments.next().is_some() {
                return Err(usage());
            }
            Ok(Invocation::PrintReleaseNotes { version })
        }
        "release-verify" => {
            let tag = arguments.next().ok_or_else(usage)?.clone();
            if arguments.next().is_some() {
                return Err(usage());
            }
            Ok(Invocation::VerifyReleaseTag { tag })
        }
        _ => Err(usage()),
    }
}

/// Returns the seconds elapsed since the Unix epoch, according to the system clock.
fn unix_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

/// Returns the repository root: the parent directory of the `xtask` crate itself.
///
/// This is resolved at compile time via `CARGO_MANIFEST_DIR`, since the runtime
/// working directory of an installed binary is not a reliable source for it.
fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        )
}

/// Runs the requested invocation against the real repository and process environment.
fn run(arguments: &[String]) -> Result<(), XtaskError> {
    match parse_invocation(arguments)? {
        Invocation::PrepareRelease {
            request,
            is_dry_run,
        } => {
            let repository_root = repository_root();
            let date = civil_date::civil_date_from_unix_seconds(unix_seconds_now());
            let mut runner = command_runner::ProcessRunner::new(repository_root.clone());
            let version = release_prep::prepare_release(
                &repository_root,
                &request,
                &date,
                is_dry_run,
                &mut runner,
            )?;
            println!("{version}");
            Ok(())
        }
        Invocation::PrintReleaseNotes { version } => {
            let changelog = std::fs::read_to_string(repository_root().join("CHANGELOG.md"))?;
            let notes = changelog::release_notes(&changelog, &version)?;
            println!("{notes}");
            Ok(())
        }
        Invocation::VerifyReleaseTag { tag } => {
            let repository_root = repository_root();
            let mut runner = command_runner::ProcessRunner::new(repository_root.clone());
            let version = release_verify::verify_release(&repository_root, &tag, &mut runner)?;
            println!("{version}");
            Ok(())
        }
    }
}

fn main() -> std::process::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
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
            Invocation::PrepareRelease {
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
            Invocation::PrepareRelease {
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
            Invocation::PrintReleaseNotes {
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

    #[test]
    fn parses_release_verify() {
        let invocation = parse_invocation(&arguments(&["release-verify", "v0.1.2"])).unwrap();
        assert_eq!(
            invocation,
            Invocation::VerifyReleaseTag {
                tag: "v0.1.2".to_string()
            }
        );
    }

    #[test]
    fn parses_release_verify_without_validating_the_tag_shape() {
        // Parsing only splits the command line; `verify_release` is the one
        // that validates the tag shape. A junk tag string still parses here.
        let invocation = parse_invocation(&arguments(&["release-verify", "not-a-tag"])).unwrap();
        assert_eq!(
            invocation,
            Invocation::VerifyReleaseTag {
                tag: "not-a-tag".to_string()
            }
        );
    }

    #[test]
    fn rejects_release_verify_without_a_tag() {
        assert!(matches!(
            parse_invocation(&arguments(&["release-verify"])),
            Err(XtaskError::Usage(_))
        ));
    }

    #[test]
    fn rejects_release_verify_with_an_extra_argument() {
        assert!(matches!(
            parse_invocation(&arguments(&["release-verify", "v0.1.2", "extra"])),
            Err(XtaskError::Usage(_))
        ));
    }
}
