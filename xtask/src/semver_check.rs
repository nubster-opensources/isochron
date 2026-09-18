//! Compares the public API with the latest release published on crates.io.
//!
//! The comparison applies the release type declared in the manifest. A `patch` declaration
//! lets the tool infer the release type from the manifest version, which is what a release
//! preparation branch needs once its version is bumped. A `minor` or `major` declaration is
//! passed explicitly: it is the reviewed permission for an intentional breaking change while
//! the manifest version still names the last release.

use crate::command_runner::CommandRunner;
use crate::error::XtaskError;
use crate::version::BumpLevel;

/// The package whose public API is compared.
const PACKAGE_NAME: &str = "isochron";

/// Returns the `cargo` arguments that compare the public API under the `declared` release type.
///
/// `--all-features` is always passed: it is inert while the crate has no feature, and it keeps
/// the comparison complete the day one is added.
pub(crate) fn semver_checks_arguments(declared: BumpLevel) -> Vec<&'static str> {
    let mut arguments = vec!["semver-checks", "--package", PACKAGE_NAME, "--all-features"];
    if declared != BumpLevel::Patch {
        arguments.push("--release-type");
        arguments.push(declared.as_str());
    }
    arguments
}

/// Reads the declared next release from the manifest of `repository_root`, then compares the
/// public API with the latest published release.
pub(crate) fn check_public_api(
    repository_root: &std::path::Path,
    runner: &mut dyn CommandRunner,
) -> Result<(), XtaskError> {
    let manifest = std::fs::read_to_string(repository_root.join("Cargo.toml"))?;
    let declared = crate::manifest::declared_next_release(&manifest)?;
    runner.run("cargo", &semver_checks_arguments(declared))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CommandRunner, check_public_api, semver_checks_arguments};
    use crate::error::XtaskError;
    use crate::version::BumpLevel;
    use std::path::{Path, PathBuf};

    const DECLARED_MINOR_MANIFEST: &str = "[package]\n\
name = \"isochron\"\n\
version = \"0.1.2\"\n\
\n\
[package.metadata.isochron]\n\
next-release = \"minor\"\n";

    /// A fake command runner that records every call and fails every call when asked to.
    struct RecordingRunner {
        calls: Vec<(String, Vec<String>)>,
        is_failing: bool,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                is_failing: false,
            }
        }

        fn failing() -> Self {
            Self {
                calls: Vec::new(),
                is_failing: true,
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&mut self, program: &str, arguments: &[&str]) -> Result<String, XtaskError> {
            self.calls.push((
                program.to_string(),
                arguments
                    .iter()
                    .map(|argument| (*argument).to_string())
                    .collect(),
            ));
            if self.is_failing {
                return Err(XtaskError::CommandFailed {
                    program: program.to_string(),
                    status: "1".to_string(),
                });
            }
            Ok(String::new())
        }
    }

    fn call(program: &str, arguments: &[&str]) -> (String, Vec<String>) {
        (
            program.to_string(),
            arguments
                .iter()
                .map(|argument| (*argument).to_string())
                .collect(),
        )
    }

    fn with_manifest_repository<T>(name: &str, manifest: &str, test: impl FnOnce(&Path) -> T) -> T {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let mut dir: PathBuf = std::env::temp_dir();
        dir.push(format!("xtask-semver-check-{name}-{nanos}"));
        std::fs::create_dir_all(&dir).expect("create temporary repository root");
        std::fs::write(dir.join("Cargo.toml"), manifest).expect("write manifest fixture");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&dir)));
        let _ = std::fs::remove_dir_all(&dir);
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    #[test]
    fn a_patch_declaration_lets_the_tool_infer_the_release_type() {
        assert_eq!(
            semver_checks_arguments(BumpLevel::Patch),
            vec!["semver-checks", "--package", "isochron", "--all-features"]
        );
    }

    #[test]
    fn a_minor_declaration_passes_the_minor_release_type() {
        assert_eq!(
            semver_checks_arguments(BumpLevel::Minor),
            vec![
                "semver-checks",
                "--package",
                "isochron",
                "--all-features",
                "--release-type",
                "minor"
            ]
        );
    }

    #[test]
    fn a_major_declaration_passes_the_major_release_type() {
        assert_eq!(
            semver_checks_arguments(BumpLevel::Major),
            vec![
                "semver-checks",
                "--package",
                "isochron",
                "--all-features",
                "--release-type",
                "major"
            ]
        );
    }

    #[test]
    fn check_public_api_runs_exactly_one_comparison_under_the_declared_release_type() {
        with_manifest_repository("declared-minor", DECLARED_MINOR_MANIFEST, |root| {
            let mut runner = RecordingRunner::new();
            check_public_api(root, &mut runner).unwrap();
            assert_eq!(
                runner.calls,
                vec![call(
                    "cargo",
                    &[
                        "semver-checks",
                        "--package",
                        "isochron",
                        "--all-features",
                        "--release-type",
                        "minor"
                    ]
                )]
            );
        });
    }

    #[test]
    fn check_public_api_refuses_a_missing_declaration_before_running_anything() {
        let manifest = "[package]\nname = \"isochron\"\nversion = \"0.1.2\"\n";
        with_manifest_repository("missing-declaration", manifest, |root| {
            let mut runner = RecordingRunner::new();
            let error = check_public_api(root, &mut runner).unwrap_err();
            assert!(matches!(error, XtaskError::NextReleaseNotFound));
            assert_eq!(runner.calls, Vec::new());
        });
    }

    #[test]
    fn check_public_api_propagates_a_failed_comparison() {
        with_manifest_repository("failed-comparison", DECLARED_MINOR_MANIFEST, |root| {
            let mut runner = RecordingRunner::failing();
            let error = check_public_api(root, &mut runner).unwrap_err();
            assert!(
                matches!(error, XtaskError::CommandFailed { program, .. } if program == "cargo")
            );
            assert_eq!(runner.calls.len(), 1);
        });
    }
}
