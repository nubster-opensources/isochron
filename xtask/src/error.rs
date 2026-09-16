//! Error type shared by every xtask command.

/// Every way an xtask command can fail.
#[derive(Debug)]
pub(crate) enum XtaskError {
    /// The changelog has no `## [Unreleased]` heading.
    MissingUnreleasedSection,
    /// A heading mentions "unreleased" but not in the exact expected form.
    MalformedUnreleasedHeading {
        /// The offending heading line, verbatim.
        line: String,
    },
    /// The `## [Unreleased]` section has no bullet line.
    EmptyUnreleasedSection,
    /// The target version already has a released heading in the changelog.
    VersionAlreadyReleased {
        /// The version that is already released.
        version: String,
    },
    /// The changelog has no `[Unreleased]: ` link reference definition.
    MissingUnreleasedLink,
    /// The requested version section is absent from the changelog.
    SectionNotFound {
        /// The version that was requested.
        version: String,
    },
    /// The requested version section has no body once trimmed.
    EmptySection {
        /// The version that was requested.
        version: String,
    },
    /// A version string does not parse as a strict `SemVer` core `x.y.z`.
    InvalidVersion {
        /// The raw input that failed to parse.
        input: String,
    },
    /// The requested version is not strictly greater than the current one.
    VersionNotGreater {
        /// The current version, formatted.
        current: String,
        /// The requested target version, formatted.
        target: String,
    },
    /// The manifest has no `version` key under `[package]`.
    ManifestVersionNotFound,
    /// The current branch is not `main`.
    NotOnMain {
        /// The branch that was checked out instead of `main`.
        branch: String,
    },
    /// The working tree has uncommitted changes.
    DirtyWorkingTree,
    /// An external command exited with a non success status.
    CommandFailed {
        /// The program that was run.
        program: String,
        /// The exit status, formatted.
        status: String,
    },
    /// A release tag is not of the form `vX.Y.Z`.
    InvalidReleaseTag {
        /// The tag that was pushed.
        tag: String,
    },
    /// The tag version and the packaged version disagree.
    TagVersionMismatch {
        /// The version claimed by the tag.
        tag_version: String,
        /// The version cargo would publish.
        package_version: String,
    },
    /// The output of `cargo pkgid` carries no readable version.
    UnreadablePackageId {
        /// The output that could not be read, verbatim.
        output: String,
    },
    /// The tag does not resolve to the commit that is checked out.
    TagNotCheckedOut {
        /// The commit the tag points at.
        tag_commit: String,
        /// The commit that is checked out.
        head_commit: String,
    },
    /// The tagged commit is not on the first-parent line of `origin/main`.
    CommitNotOnMainLine {
        /// The commit the tag points at.
        commit: String,
    },
    /// An underlying input and output error.
    Io(std::io::Error),
    /// The command line arguments could not be interpreted.
    Usage(String),
}

impl std::fmt::Display for XtaskError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUnreleasedSection => write!(
                formatter,
                "CHANGELOG.md has no `## [Unreleased]` heading; add one above the latest release"
            ),
            Self::MalformedUnreleasedHeading { line } => write!(
                formatter,
                "CHANGELOG.md has a heading that looks like the unreleased section but is not exactly `## [Unreleased]`: `{line}`; rename it to `## [Unreleased]`"
            ),
            Self::EmptyUnreleasedSection => write!(
                formatter,
                "The `## [Unreleased]` section in CHANGELOG.md has no content; add at least one bullet before preparing a release"
            ),
            Self::VersionAlreadyReleased { version } => write!(
                formatter,
                "Version `{version}` already has a released section in CHANGELOG.md; choose a different version"
            ),
            Self::MissingUnreleasedLink => write!(
                formatter,
                "CHANGELOG.md has no `[Unreleased]: ` link reference definition; add one so the compare link can be updated"
            ),
            Self::SectionNotFound { version } => write!(
                formatter,
                "CHANGELOG.md has no `## [{version}]` section; check the version number"
            ),
            Self::EmptySection { version } => write!(
                formatter,
                "The `## [{version}]` section in CHANGELOG.md has no content"
            ),
            Self::InvalidVersion { input } => write!(
                formatter,
                "`{input}` is not a valid version; expected a strict SemVer core `x.y.z` with no leading zeros, prerelease or build metadata"
            ),
            Self::VersionNotGreater { current, target } => write!(
                formatter,
                "Requested version `{target}` is not strictly greater than the current version `{current}`; choose a higher version"
            ),
            Self::ManifestVersionNotFound => write!(
                formatter,
                "Cargo.toml has no `version` key under `[package]`; add one before running this command"
            ),
            Self::NotOnMain { branch } => write!(
                formatter,
                "Must be on branch `main` to prepare a release, but the current branch is `{branch}`; switch to `main` first"
            ),
            Self::DirtyWorkingTree => write!(
                formatter,
                "The working tree has uncommitted changes; commit or stash them before preparing a release"
            ),
            Self::CommandFailed { program, status } => write!(
                formatter,
                "Command `{program}` failed with exit status {status}; check its output above for details"
            ),
            Self::InvalidReleaseTag { tag } => write!(
                formatter,
                "Release tag `{tag}` is not of the form `vX.Y.Z` with no prerelease or build metadata; delete the tag and push a corrected one"
            ),
            Self::TagVersionMismatch {
                tag_version,
                package_version,
            } => write!(
                formatter,
                "Tag version `{tag_version}` does not match the packaged version `{package_version}`; the tag was pushed for a version this commit does not carry"
            ),
            Self::UnreadablePackageId { output } => write!(
                formatter,
                "Could not read a version from the `cargo pkgid` output `{output}`"
            ),
            Self::TagNotCheckedOut {
                tag_commit,
                head_commit,
            } => write!(
                formatter,
                "The tag points at commit `{tag_commit}` but commit `{head_commit}` is checked out; release from the tagged commit only"
            ),
            Self::CommitNotOnMainLine { commit } => write!(
                formatter,
                "Commit `{commit}` is not on the first-parent line of `origin/main`; only a commit that `main` itself points at may be released"
            ),
            Self::Io(error) => write!(formatter, "input and output error: {error}"),
            Self::Usage(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for XtaskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for XtaskError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
