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
