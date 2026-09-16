//! Automates the release preparation workflow: branch and dirty tree checks,
//! changelog graduation, manifest bump, commit, local verification, and pull request.

use crate::command_runner::CommandRunner;
use crate::error::XtaskError;
use crate::version::{Version, VersionRequest};

/// Builds the release branch name for `version`, `chore/release-vX.Y.Z`.
pub(crate) fn branch_name(version: &Version) -> String {
    format!("chore/release-v{version}")
}

/// Builds the pull request body explaining what happens after this pull request merges.
fn release_pull_request_body(version: &Version) -> String {
    format!(
        "This pull request bumps isochron to {version} and graduates the changelog.\n\
\n\
After this pull request is merged, push the annotated tag `v{version}` to trigger the Release workflow:\n\
\n\
```\n\
git tag -a v{version} -m \"v{version}\"\n\
git push origin v{version}\n\
```\n\
\n\
The Release workflow then publishes the crate to crates.io and creates the GitHub Release."
    )
}

/// Runs the full release preparation workflow against `repository_root`.
pub(crate) fn prepare_release(
    repository_root: &std::path::Path,
    request: &VersionRequest,
    date: &str,
    is_dry_run: bool,
    runner: &mut dyn CommandRunner,
) -> Result<Version, XtaskError> {
    let current_branch = runner.run("git", &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let current_branch = current_branch.trim();
    if current_branch != "main" {
        return Err(XtaskError::NotOnMain {
            branch: current_branch.to_string(),
        });
    }

    let status = runner.run("git", &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(XtaskError::DirtyWorkingTree);
    }

    runner.run("git", &["pull", "--ff-only", "origin", "main"])?;

    let manifest_path = repository_root.join("Cargo.toml");
    let changelog_path = repository_root.join("CHANGELOG.md");
    let manifest = std::fs::read_to_string(&manifest_path)?;
    let changelog = std::fs::read_to_string(&changelog_path)?;

    let current_version = crate::manifest::package_version(&manifest)?;
    let target_version = request.resolve(&current_version)?;
    let graduated_changelog = crate::changelog::graduate(&changelog, &target_version, date)?;
    let updated_manifest = crate::manifest::with_package_version(&manifest, &target_version)?;

    let branch = branch_name(&target_version);
    runner.run("git", &["checkout", "-b", &branch])?;

    std::fs::write(&manifest_path, &updated_manifest)?;
    std::fs::write(&changelog_path, &graduated_changelog)?;

    runner.run("git", &["add", "CHANGELOG.md", "Cargo.toml"])?;
    let commit_message = format!(
        "release(v{target_version}): bump isochron to {target_version} and graduate the changelog"
    );
    runner.run("git", &["commit", "-m", &commit_message])?;

    runner.run("cargo", &["fmt", "--all", "--check"])?;
    runner.run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    runner.run("cargo", &["test", "--workspace", "--all-features"])?;

    if !is_dry_run {
        runner.run("git", &["push", "-u", "origin", &branch])?;

        let title = format!("release(v{target_version}): bump isochron and graduate the changelog");
        let body = release_pull_request_body(&target_version);
        runner.run(
            "gh",
            &[
                "pr", "create", "--base", "main", "--head", &branch, "--title", &title, "--body",
                &body,
            ],
        )?;
    }

    Ok(target_version)
}

#[cfg(test)]
mod tests {
    use super::{CommandRunner, prepare_release};
    use crate::error::XtaskError;
    use crate::version::{BumpLevel, VersionRequest};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    const FIXTURE_MANIFEST: &str = "[package]\nname = \"isochron\"\nversion = \"0.1.1\"\n";
    const FIXTURE_CHANGELOG: &str = "## [Unreleased]\n\n\
### Added\n\n\
- Something pending.\n\n\
## [0.1.1] - 2026-06-18\n\n\
### Changed\n\n\
- Older change.\n\n\
[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD\n\
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1\n";
    const INVALID_CHANGELOG: &str = "# No unreleased section here\n";

    /// A fake command runner that records every call and returns configured
    /// outputs or failures by call index, in the order `run` is invoked.
    struct RecordingRunner {
        calls: Vec<(String, Vec<String>)>,
        outputs: HashMap<usize, String>,
        fail_at: Option<usize>,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                outputs: HashMap::new(),
                fail_at: None,
            }
        }

        fn with_output(mut self, call_index: usize, output: &str) -> Self {
            self.outputs.insert(call_index, output.to_string());
            self
        }

        fn failing_at(mut self, call_index: usize) -> Self {
            self.fail_at = Some(call_index);
            self
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&mut self, program: &str, arguments: &[&str]) -> Result<String, XtaskError> {
            let index = self.calls.len();
            self.calls.push((
                program.to_string(),
                arguments
                    .iter()
                    .map(|argument| (*argument).to_string())
                    .collect(),
            ));
            if self.fail_at == Some(index) {
                return Err(XtaskError::CommandFailed {
                    program: program.to_string(),
                    status: "1".to_string(),
                });
            }
            Ok(self.outputs.get(&index).cloned().unwrap_or_default())
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

    /// Exact runner calls, program and every argument, up to and including the
    /// local verification steps, for a release of 0.1.2.
    fn expected_calls_up_to_verification() -> Vec<(String, Vec<String>)> {
        vec![
            call("git", &["rev-parse", "--abbrev-ref", "HEAD"]),
            call("git", &["status", "--porcelain"]),
            call("git", &["pull", "--ff-only", "origin", "main"]),
            call("git", &["checkout", "-b", "chore/release-v0.1.2"]),
            call("git", &["add", "CHANGELOG.md", "Cargo.toml"]),
            call(
                "git",
                &[
                    "commit",
                    "-m",
                    "release(v0.1.2): bump isochron to 0.1.2 and graduate the changelog",
                ],
            ),
            call("cargo", &["fmt", "--all", "--check"]),
            call(
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            call("cargo", &["test", "--workspace", "--all-features"]),
        ]
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let mut dir = std::env::temp_dir();
        dir.push(format!("xtask-release-prep-{name}-{nanos}"));
        dir
    }

    fn with_fixture_repository<T>(
        name: &str,
        manifest: &str,
        changelog: &str,
        test: impl FnOnce(&Path) -> T,
    ) -> T {
        let dir = unique_temp_dir(name);
        std::fs::create_dir_all(&dir).expect("create temporary repository root");
        std::fs::write(dir.join("Cargo.toml"), manifest).expect("write manifest fixture");
        std::fs::write(dir.join("CHANGELOG.md"), changelog).expect("write changelog fixture");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&dir)));
        let _ = std::fs::remove_dir_all(&dir);
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    #[test]
    fn dry_run_records_the_verification_steps_without_pushing_or_opening_a_pull_request() {
        with_fixture_repository("dry-run", FIXTURE_MANIFEST, FIXTURE_CHANGELOG, |root| {
            let mut runner = RecordingRunner::new().with_output(0, "main\n");
            let target = prepare_release(
                root,
                &VersionRequest::Level(BumpLevel::Patch),
                "2026-09-15",
                true,
                &mut runner,
            )
            .unwrap();
            assert_eq!(target.to_string(), "0.1.2");
            assert_eq!(runner.calls, expected_calls_up_to_verification());
            assert!(!runner.calls.iter().any(|(program, arguments)| {
                program == "git" && arguments.first().map(String::as_str) == Some("push")
            }));
            assert!(!runner.calls.iter().any(|(program, _)| program == "gh"));

            let manifest_after = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
            assert_eq!(
                manifest_after,
                "[package]\nname = \"isochron\"\nversion = \"0.1.2\"\n"
            );
            let changelog_after = std::fs::read_to_string(root.join("CHANGELOG.md")).unwrap();
            assert!(changelog_after.contains("## [Unreleased]\n\n## [0.1.2] - 2026-09-15\n"));
            assert!(changelog_after.contains(
                "[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.2...HEAD\n\
[0.1.2]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...v0.1.2\n"
            ));
        });
    }

    #[test]
    fn not_dry_run_pushes_and_opens_a_pull_request_after_verification() {
        with_fixture_repository("not-dry-run", FIXTURE_MANIFEST, FIXTURE_CHANGELOG, |root| {
            let mut runner = RecordingRunner::new().with_output(0, "main\n");
            prepare_release(
                root,
                &VersionRequest::Level(BumpLevel::Patch),
                "2026-09-15",
                false,
                &mut runner,
            )
            .unwrap();
            assert_eq!(runner.calls.len(), 11);
            assert_eq!(runner.calls[..9], expected_calls_up_to_verification()[..]);
            assert_eq!(
                runner.calls[9],
                call("git", &["push", "-u", "origin", "chore/release-v0.1.2"])
            );
            assert_eq!(runner.calls[10].0, "gh");
            let pull_request_arguments = &runner.calls[10].1;
            for expected in ["--base", "main", "--head", "chore/release-v0.1.2"] {
                assert!(
                    pull_request_arguments
                        .iter()
                        .any(|argument| argument == expected),
                    "gh pr create is missing {expected}"
                );
            }
            assert_eq!(runner.calls[10].1.first().map(String::as_str), Some("pr"));
            assert_eq!(
                runner.calls[10].1.get(1).map(String::as_str),
                Some("create")
            );
        });
    }

    #[test]
    fn invalid_changelog_stops_after_the_dirty_tree_check() {
        with_fixture_repository(
            "invalid-changelog",
            FIXTURE_MANIFEST,
            INVALID_CHANGELOG,
            |root| {
                let mut runner = RecordingRunner::new().with_output(0, "main\n");
                let error = prepare_release(
                    root,
                    &VersionRequest::Level(BumpLevel::Patch),
                    "2026-09-15",
                    true,
                    &mut runner,
                )
                .unwrap_err();
                assert!(matches!(error, XtaskError::MissingUnreleasedSection));
                assert_eq!(runner.calls[..], expected_calls_up_to_verification()[..3]);
                let changelog_after = std::fs::read_to_string(root.join("CHANGELOG.md")).unwrap();
                assert_eq!(changelog_after, INVALID_CHANGELOG);
                let manifest_after = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
                assert_eq!(manifest_after, FIXTURE_MANIFEST);
            },
        );
    }

    #[test]
    fn refuses_to_run_outside_main() {
        with_fixture_repository("not-on-main", FIXTURE_MANIFEST, FIXTURE_CHANGELOG, |root| {
            let mut runner = RecordingRunner::new().with_output(0, "feature/other\n");
            let error = prepare_release(
                root,
                &VersionRequest::Level(BumpLevel::Patch),
                "2026-09-15",
                true,
                &mut runner,
            )
            .unwrap_err();
            assert!(matches!(error, XtaskError::NotOnMain { branch } if branch == "feature/other"));
            assert_eq!(runner.calls.len(), 1);
        });
    }

    #[test]
    fn refuses_to_run_with_a_dirty_working_tree() {
        with_fixture_repository("dirty-tree", FIXTURE_MANIFEST, FIXTURE_CHANGELOG, |root| {
            let mut runner = RecordingRunner::new()
                .with_output(0, "main\n")
                .with_output(1, " M src/lib.rs\n");
            let error = prepare_release(
                root,
                &VersionRequest::Level(BumpLevel::Patch),
                "2026-09-15",
                true,
                &mut runner,
            )
            .unwrap_err();
            assert!(matches!(error, XtaskError::DirtyWorkingTree));
            assert_eq!(runner.calls.len(), 2);
        });
    }

    #[test]
    fn dry_run_commits_exactly_once() {
        with_fixture_repository(
            "single-commit",
            FIXTURE_MANIFEST,
            FIXTURE_CHANGELOG,
            |root| {
                let mut runner = RecordingRunner::new().with_output(0, "main\n");
                prepare_release(
                    root,
                    &VersionRequest::Level(BumpLevel::Patch),
                    "2026-09-15",
                    true,
                    &mut runner,
                )
                .unwrap();
                let commit_calls = runner
                    .calls
                    .iter()
                    .filter(|(program, arguments)| {
                        program == "git" && arguments.first().map(String::as_str) == Some("commit")
                    })
                    .count();
                assert_eq!(commit_calls, 1);
            },
        );
    }

    #[test]
    fn a_failure_during_local_verification_stops_before_running_the_tests() {
        with_fixture_repository(
            "clippy-failure",
            FIXTURE_MANIFEST,
            FIXTURE_CHANGELOG,
            |root| {
                let mut runner = RecordingRunner::new()
                    .with_output(0, "main\n")
                    .failing_at(7);
                let error = prepare_release(
                    root,
                    &VersionRequest::Level(BumpLevel::Patch),
                    "2026-09-15",
                    true,
                    &mut runner,
                )
                .unwrap_err();
                assert!(matches!(error, XtaskError::CommandFailed { .. }));
                assert_eq!(runner.calls[..], expected_calls_up_to_verification()[..8]);
            },
        );
    }

    #[test]
    fn branch_name_uses_the_chore_release_prefix() {
        assert_eq!(
            super::branch_name(&crate::version::Version::new(0, 1, 2)),
            "chore/release-v0.1.2"
        );
    }
}
