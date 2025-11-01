// Checks for external prerequisites needed by the plagiarism checker

use std::process::{Command, Stdio};

/// Verify that the required external tool `copydetect` is available on PATH.
///
/// Returns whether if the command can be spawned.
pub fn check_prereq() -> bool {
    let try_spawn = Command::new("copydetect")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    try_spawn.is_ok()
}
