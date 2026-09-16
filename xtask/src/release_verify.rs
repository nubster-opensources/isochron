//! Verifies that a release tag may be published, before any step that can reach crates.io.
//!
//! Publication is irreversible: a version can be yanked, never deleted nor reused. Every
//! check here therefore runs before the publishing job receives any credential, and the
//! first failure stops the sequence.

use crate::command_runner::CommandRunner;
use crate::error::XtaskError;
use crate::version::Version;

/// The package whose manifest version a release tag must match.
const PACKAGE_NAME: &str = "isochron";

/// Parses a release tag into the version it claims to publish.
///
/// The accepted shape is `v` followed by a strict `SemVer` core `x.y.z`, the same grammar
/// the release preparation uses, so that both ends of the chain agree on what a version is.
/// Prereleases and build metadata are refused: crates.io ignores build metadata, which would
/// make `v0.1.2+a` and `v0.1.2` two names for one published version.
pub(crate) fn tag_version(tag: &str) -> Result<Version, XtaskError> {
    let _ = tag;
    todo!("parse a release tag into its version")
}

/// Reads the package version out of the output of `cargo pkgid`.
///
/// Cargo prints either `...#0.1.2`, when the directory is named after the package, or
/// `...#isochron@0.1.2` otherwise. Both forms are accepted, because the checkout directory
/// name is not something a release may depend on.
pub(crate) fn package_version_from_pkgid(pkgid_output: &str) -> Result<Version, XtaskError> {
    let _ = pkgid_output;
    todo!("read the package version from a cargo pkgid output")
}

/// Whether `commit` appears as a whole line of a `git rev-list --first-parent` listing.
///
/// The comparison is exact: an abbreviated identifier must not match a full one, otherwise a
/// short prefix would be enough to claim membership of the released history.
pub(crate) fn is_on_first_parent_line(commit: &str, first_parent_listing: &str) -> bool {
    let _ = (commit, first_parent_listing);
    todo!("decide whether a commit is on the first-parent line")
}

/// Verifies that `tag` may be published from the currently checked out commit, and returns
/// the version it publishes.
///
/// The checks run in this order, each one stopping the sequence on failure:
///
/// 1. `tag` parses as `vX.Y.Z`, before any command runs, so that no unvalidated string is
///    ever handed to `git` as an argument;
/// 2. `cargo pkgid --package isochron` reports exactly the same version;
/// 3. `CHANGELOG.md` has a non empty section for that version, so that the release notes
///    cannot fail after the crate has been published;
/// 4. the tag resolves to the commit that is checked out;
/// 5. `origin/main` is refreshed;
/// 6. that commit is on the first-parent line of `origin/main`, which holds the states that
///    the continuous integration validated as `main`, and not the intermediate commits of a
///    merged branch.
pub(crate) fn verify_release(
    repository_root: &std::path::Path,
    tag: &str,
    runner: &mut dyn CommandRunner,
) -> Result<Version, XtaskError> {
    let _ = (repository_root, tag, runner, PACKAGE_NAME);
    todo!("verify a release tag")
}

#[cfg(test)]
mod tests {
    use super::{
        CommandRunner, is_on_first_parent_line, package_version_from_pkgid, tag_version,
        verify_release,
    };
    use crate::error::XtaskError;
    use crate::version::Version;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    // -- tag_version -----------------------------------------------------

    #[test]
    fn tag_version_accepts_a_simple_patch_tag() {
        assert_eq!(tag_version("v0.1.2").unwrap(), Version::new(0, 1, 2));
    }

    #[test]
    fn tag_version_accepts_multi_digit_components() {
        assert_eq!(tag_version("v10.20.30").unwrap(), Version::new(10, 20, 30));
    }

    #[test]
    fn tag_version_refuses_a_tag_without_the_v_prefix() {
        let error = tag_version("0.1.2").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { tag } if tag == "0.1.2"));
    }

    #[test]
    fn tag_version_refuses_an_uppercase_v_prefix() {
        let error = tag_version("V0.1.2").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_a_doubled_v_prefix() {
        let error = tag_version("vv0.1.2").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_a_missing_patch_component() {
        let error = tag_version("v0.1").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_an_extra_component() {
        let error = tag_version("v0.1.2.3").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_a_leading_zero_component() {
        let error = tag_version("v01.1.2").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_a_prerelease_suffix() {
        let error = tag_version("v0.1.2-rc.1").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_build_metadata() {
        let error = tag_version("v0.1.2+build").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_an_empty_tag() {
        let error = tag_version("").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    #[test]
    fn tag_version_refuses_a_bare_v() {
        let error = tag_version("v").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
    }

    // -- package_version_from_pkgid --------------------------------------

    #[test]
    fn package_version_from_pkgid_reads_the_bare_version_form() {
        let version =
            package_version_from_pkgid("path+file:///home/runner/work/isochron/isochron#0.1.2")
                .unwrap();
        assert_eq!(version, Version::new(0, 1, 2));
    }

    #[test]
    fn package_version_from_pkgid_reads_the_name_at_version_form() {
        let version =
            package_version_from_pkgid("path+file:///tmp/checkout#isochron@0.1.2").unwrap();
        assert_eq!(version, Version::new(0, 1, 2));
    }

    #[test]
    fn package_version_from_pkgid_tolerates_a_trailing_newline() {
        let version =
            package_version_from_pkgid("path+file:///home/runner/work/isochron/isochron#0.1.2\n")
                .unwrap();
        assert_eq!(version, Version::new(0, 1, 2));
    }

    #[test]
    fn package_version_from_pkgid_refuses_an_output_with_no_hash() {
        let error = package_version_from_pkgid("path+file:///home/runner/work/isochron/isochron")
            .unwrap_err();
        assert!(matches!(error, XtaskError::UnreadablePackageId { .. }));
    }

    #[test]
    fn package_version_from_pkgid_refuses_a_malformed_version_part() {
        // Choice made here: a malformed version part, once the `#` marker is
        // found, is reported as `InvalidVersion`, not `UnreadablePackageId`.
        // Reading the version out of a `pkgid` output naturally delegates to
        // `Version::from_str` for the part after `#` (or after `@`); that
        // parse failure is `InvalidVersion`, the same error every other
        // strict version parse in this workspace produces.
        // `UnreadablePackageId` is reserved for the case where there is
        // nothing to even attempt to parse: no `#` separator at all.
        let error =
            package_version_from_pkgid("path+file:///tmp/checkout#isochron@0.1").unwrap_err();
        assert!(matches!(error, XtaskError::InvalidVersion { .. }));
    }

    // -- is_on_first_parent_line ------------------------------------------

    const COMMIT_ONE: &str = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    const COMMIT_TWO: &str = "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222";
    const COMMIT_THREE: &str = "cccc3333cccc3333cccc3333cccc3333cccc3333";
    const COMMIT_ABSENT: &str = "dddd4444dddd4444dddd4444dddd4444dddd4444";

    fn three_commit_listing() -> String {
        format!("{COMMIT_ONE}\n{COMMIT_TWO}\n{COMMIT_THREE}")
    }

    #[test]
    fn is_on_first_parent_line_true_for_the_first_line() {
        assert!(is_on_first_parent_line(COMMIT_ONE, &three_commit_listing()));
    }

    #[test]
    fn is_on_first_parent_line_true_for_a_middle_line() {
        assert!(is_on_first_parent_line(COMMIT_TWO, &three_commit_listing()));
    }

    #[test]
    fn is_on_first_parent_line_true_for_the_last_line_without_a_trailing_newline() {
        assert!(is_on_first_parent_line(
            COMMIT_THREE,
            &three_commit_listing()
        ));
    }

    #[test]
    fn is_on_first_parent_line_true_for_the_last_line_with_a_trailing_newline() {
        let listing = format!("{}\n", three_commit_listing());
        assert!(is_on_first_parent_line(COMMIT_THREE, &listing));
    }

    #[test]
    fn is_on_first_parent_line_false_when_the_commit_is_absent() {
        assert!(!is_on_first_parent_line(
            COMMIT_ABSENT,
            &three_commit_listing()
        ));
    }

    #[test]
    fn is_on_first_parent_line_false_when_the_argument_is_an_abbreviation_of_a_listed_commit() {
        let abbreviation = &COMMIT_ONE[..7];
        assert!(!is_on_first_parent_line(
            abbreviation,
            &three_commit_listing()
        ));
    }

    #[test]
    fn is_on_first_parent_line_false_when_the_listed_commit_is_an_abbreviation_of_the_argument() {
        let listing = format!("{}\n{COMMIT_TWO}\n{COMMIT_THREE}", &COMMIT_ONE[..7]);
        assert!(!is_on_first_parent_line(COMMIT_ONE, &listing));
    }

    // -- verify_release -----------------------------------------------------

    const TAG: &str = "v0.1.2";
    const PKGID_MATCHING: &str = "path+file:///tmp/isochron#isochron@0.1.2\n";
    const PKGID_MISMATCHED: &str = "path+file:///tmp/isochron#isochron@0.1.3\n";
    const VERIFY_COMMIT_A: &str = "a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1";
    const VERIFY_COMMIT_B: &str = "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2";
    const CHANGELOG_WITH_SECTION: &str = "## [Unreleased]\n\n\
- Pending change.\n\n\
## [0.1.2] - 2026-09-15\n\n\
### Added\n\n\
- Something released.\n\n\
[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.2...HEAD\n\
[0.1.2]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.2\n";
    const CHANGELOG_WITH_EMPTY_SECTION: &str = "## [Unreleased]\n\n\
- Pending change.\n\n\
## [0.1.2] - 2026-09-15\n\n\
## [0.1.1] - 2026-06-18\n\n\
### Changed\n\n\
- Older change.\n\n\
[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.2...HEAD\n";
    const CHANGELOG_WITHOUT_SECTION: &str = "## [Unreleased]\n\n\
- Pending change.\n\n\
## [0.1.1] - 2026-06-18\n\n\
### Changed\n\n\
- Older change.\n\n\
[Unreleased]: https://github.com/nubster-opensources/isochron/compare/v0.1.1...HEAD\n\
[0.1.1]: https://github.com/nubster-opensources/isochron/releases/tag/v0.1.1\n";

    /// A fake command runner that records every call and returns a canned
    /// output (or failure) keyed on the exact program and argument list, so
    /// that two commands can never be swapped without a test noticing.
    struct KeyedRunner {
        calls: Vec<(String, Vec<String>)>,
        outputs: HashMap<(String, Vec<String>), String>,
        fail_on: Option<(String, Vec<String>)>,
    }

    impl KeyedRunner {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                outputs: HashMap::new(),
                fail_on: None,
            }
        }

        fn with_output(mut self, program: &str, arguments: &[&str], output: &str) -> Self {
            self.outputs
                .insert(call(program, arguments), output.to_string());
            self
        }

        fn failing_on(mut self, program: &str, arguments: &[&str]) -> Self {
            self.fail_on = Some(call(program, arguments));
            self
        }
    }

    impl CommandRunner for KeyedRunner {
        fn run(&mut self, program: &str, arguments: &[&str]) -> Result<String, XtaskError> {
            let key = call(program, arguments);
            self.calls.push(key.clone());
            if self.fail_on.as_ref() == Some(&key) {
                return Err(XtaskError::CommandFailed {
                    program: program.to_string(),
                    status: "1".to_string(),
                });
            }
            Ok(self.outputs.get(&key).cloned().unwrap_or_default())
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

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let mut dir = std::env::temp_dir();
        dir.push(format!("xtask-release-verify-{name}-{nanos}"));
        dir
    }

    fn with_changelog_repository<T>(
        name: &str,
        changelog: &str,
        test: impl FnOnce(&Path) -> T,
    ) -> T {
        let dir = unique_temp_dir(name);
        std::fs::create_dir_all(&dir).expect("create temporary repository root");
        std::fs::write(dir.join("CHANGELOG.md"), changelog).expect("write changelog fixture");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&dir)));
        let _ = std::fs::remove_dir_all(&dir);
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    #[test]
    fn verify_release_happy_path_returns_the_version_and_calls_are_exact() {
        with_changelog_repository("happy-path", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new()
                .with_output("cargo", &["pkgid", "--package", "isochron"], PKGID_MATCHING)
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "HEAD"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &[
                        "fetch",
                        "--no-tags",
                        "origin",
                        "+refs/heads/main:refs/remotes/origin/main",
                    ],
                    "",
                )
                .with_output(
                    "git",
                    &["rev-list", "--first-parent", "refs/remotes/origin/main"],
                    &format!("{VERIFY_COMMIT_A}\n{VERIFY_COMMIT_B}\n"),
                );

            let version = verify_release(root, TAG, &mut runner).unwrap();
            assert_eq!(version, Version::new(0, 1, 2));
            assert_eq!(
                runner.calls,
                vec![
                    call("cargo", &["pkgid", "--package", "isochron"]),
                    call(
                        "git",
                        &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    ),
                    call("git", &["rev-parse", "--verify", "HEAD"]),
                    call(
                        "git",
                        &[
                            "fetch",
                            "--no-tags",
                            "origin",
                            "+refs/heads/main:refs/remotes/origin/main",
                        ],
                    ),
                    call(
                        "git",
                        &["rev-list", "--first-parent", "refs/remotes/origin/main"],
                    ),
                ]
            );
        });
    }

    #[test]
    fn verify_release_refuses_an_invalid_tag_before_any_command_runs() {
        with_changelog_repository("invalid-tag", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new();
            let error = verify_release(root, "v0.1.2-rc.1", &mut runner).unwrap_err();
            assert!(matches!(error, XtaskError::InvalidReleaseTag { .. }));
            assert_eq!(runner.calls, Vec::new());
        });
    }

    #[test]
    fn verify_release_reports_a_tag_version_mismatch_after_only_the_pkgid_call() {
        with_changelog_repository("version-mismatch", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new().with_output(
                "cargo",
                &["pkgid", "--package", "isochron"],
                PKGID_MISMATCHED,
            );
            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            assert!(matches!(
                error,
                XtaskError::TagVersionMismatch {
                    tag_version,
                    package_version,
                } if tag_version == "0.1.2" && package_version == "0.1.3"
            ));
            assert_eq!(
                runner.calls,
                vec![call("cargo", &["pkgid", "--package", "isochron"])]
            );
        });
    }

    #[test]
    fn verify_release_reports_a_missing_changelog_section_without_any_git_call() {
        with_changelog_repository("missing-section", CHANGELOG_WITHOUT_SECTION, |root| {
            let mut runner = KeyedRunner::new().with_output(
                "cargo",
                &["pkgid", "--package", "isochron"],
                PKGID_MATCHING,
            );
            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            assert!(matches!(error, XtaskError::SectionNotFound { version } if version == "0.1.2"));
            assert_eq!(
                runner.calls,
                vec![call("cargo", &["pkgid", "--package", "isochron"])]
            );
        });
    }

    #[test]
    fn verify_release_reports_when_the_tag_commit_differs_from_head_and_stops_before_fetch() {
        with_changelog_repository("tag-not-checked-out", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new()
                .with_output("cargo", &["pkgid", "--package", "isochron"], PKGID_MATCHING)
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "HEAD"],
                    &format!("{VERIFY_COMMIT_B}\n"),
                );

            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            // The reported identifiers carry no surrounding whitespace: the
            // message names commits, not the raw bytes git happened to print.
            assert!(matches!(
                error,
                XtaskError::TagNotCheckedOut {
                    tag_commit,
                    head_commit,
                } if tag_commit == VERIFY_COMMIT_A && head_commit == VERIFY_COMMIT_B
            ));
            assert_eq!(
                runner.calls,
                vec![
                    call("cargo", &["pkgid", "--package", "isochron"]),
                    call(
                        "git",
                        &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    ),
                    call("git", &["rev-parse", "--verify", "HEAD"]),
                ]
            );
        });
    }

    #[test]
    fn verify_release_reports_when_the_tagged_commit_is_absent_from_the_first_parent_line() {
        with_changelog_repository("commit-not-on-main-line", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new()
                .with_output("cargo", &["pkgid", "--package", "isochron"], PKGID_MATCHING)
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "HEAD"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &[
                        "fetch",
                        "--no-tags",
                        "origin",
                        "+refs/heads/main:refs/remotes/origin/main",
                    ],
                    "",
                )
                .with_output(
                    "git",
                    &["rev-list", "--first-parent", "refs/remotes/origin/main"],
                    &format!("{VERIFY_COMMIT_B}\n"),
                );

            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            assert!(matches!(
                error,
                XtaskError::CommitNotOnMainLine { commit } if commit == VERIFY_COMMIT_A
            ));
            assert_eq!(
                runner.calls,
                vec![
                    call("cargo", &["pkgid", "--package", "isochron"]),
                    call(
                        "git",
                        &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    ),
                    call("git", &["rev-parse", "--verify", "HEAD"]),
                    call(
                        "git",
                        &[
                            "fetch",
                            "--no-tags",
                            "origin",
                            "+refs/heads/main:refs/remotes/origin/main",
                        ],
                    ),
                    call(
                        "git",
                        &["rev-list", "--first-parent", "refs/remotes/origin/main"],
                    ),
                ]
            );
        });
    }

    #[test]
    fn verify_release_refuses_an_empty_changelog_section_without_any_git_call() {
        with_changelog_repository("empty-section", CHANGELOG_WITH_EMPTY_SECTION, |root| {
            let mut runner = KeyedRunner::new().with_output(
                "cargo",
                &["pkgid", "--package", "isochron"],
                PKGID_MATCHING,
            );
            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            assert!(matches!(error, XtaskError::EmptySection { version } if version == "0.1.2"));
            assert_eq!(
                runner.calls,
                vec![call("cargo", &["pkgid", "--package", "isochron"])]
            );
        });
    }

    #[test]
    fn verify_release_compares_commits_ignoring_line_endings() {
        // git output reaches us verbatim, and a carriage return must not make
        // two identical commits look different.
        with_changelog_repository("carriage-returns", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new()
                .with_output("cargo", &["pkgid", "--package", "isochron"], PKGID_MATCHING)
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    &format!("{VERIFY_COMMIT_A}\r\n"),
                )
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "HEAD"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &[
                        "fetch",
                        "--no-tags",
                        "origin",
                        "+refs/heads/main:refs/remotes/origin/main",
                    ],
                    "",
                )
                .with_output(
                    "git",
                    &["rev-list", "--first-parent", "refs/remotes/origin/main"],
                    &format!("{VERIFY_COMMIT_A}\r\n{VERIFY_COMMIT_B}\r\n"),
                );

            let version = verify_release(root, TAG, &mut runner).unwrap();
            assert_eq!(version, Version::new(0, 1, 2));
        });
    }

    #[test]
    fn verify_release_propagates_a_command_failure_unchanged() {
        with_changelog_repository("command-failure", CHANGELOG_WITH_SECTION, |root| {
            let mut runner = KeyedRunner::new()
                .with_output("cargo", &["pkgid", "--package", "isochron"], PKGID_MATCHING)
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .with_output(
                    "git",
                    &["rev-parse", "--verify", "HEAD"],
                    &format!("{VERIFY_COMMIT_A}\n"),
                )
                .failing_on(
                    "git",
                    &[
                        "fetch",
                        "--no-tags",
                        "origin",
                        "+refs/heads/main:refs/remotes/origin/main",
                    ],
                );

            let error = verify_release(root, TAG, &mut runner).unwrap_err();
            assert!(matches!(error, XtaskError::CommandFailed { .. }));
            assert_eq!(
                runner.calls,
                vec![
                    call("cargo", &["pkgid", "--package", "isochron"]),
                    call(
                        "git",
                        &["rev-parse", "--verify", "refs/tags/v0.1.2^{commit}"],
                    ),
                    call("git", &["rev-parse", "--verify", "HEAD"]),
                    call(
                        "git",
                        &[
                            "fetch",
                            "--no-tags",
                            "origin",
                            "+refs/heads/main:refs/remotes/origin/main",
                        ],
                    ),
                ]
            );
        });
    }
}
