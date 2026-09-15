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
    /// A version string does not parse as a strict SemVer core `x.y.z`.
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
    /// An underlying input and output error.
    Io(std::io::Error),
    /// The command line arguments could not be interpreted.
    Usage(String),
}

impl std::fmt::Display for XtaskError {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for XtaskError {}

impl From<std::io::Error> for XtaskError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
