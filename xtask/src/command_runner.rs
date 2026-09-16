//! Runs external commands, behind a trait so that command sequences stay testable.

use crate::error::XtaskError;

/// Runs an external command and returns its captured standard output.
pub(crate) trait CommandRunner {
    /// Runs `program` with `arguments` in the repository root and returns its standard output.
    fn run(&mut self, program: &str, arguments: &[&str]) -> Result<String, XtaskError>;
}

/// Runs external commands as real OS processes, rooted at `repository_root`.
pub(crate) struct ProcessRunner {
    /// The directory every spawned process runs in.
    repository_root: std::path::PathBuf,
}

impl ProcessRunner {
    /// Creates a process runner that executes every command inside `repository_root`.
    pub(crate) fn new(repository_root: std::path::PathBuf) -> Self {
        Self { repository_root }
    }
}

impl CommandRunner for ProcessRunner {
    fn run(&mut self, program: &str, arguments: &[&str]) -> Result<String, XtaskError> {
        let output = std::process::Command::new(program)
            .args(arguments)
            .current_dir(&self.repository_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .output()?;

        // Forward the captured standard output to our own, so CI logs still
        // show the wrapped command's output, in addition to returning it.
        {
            use std::io::Write as _;
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(&output.stdout);
            let _ = stdout.flush();
        }

        if !output.status.success() {
            return Err(XtaskError::CommandFailed {
                program: program.to_string(),
                status: output.status.to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
