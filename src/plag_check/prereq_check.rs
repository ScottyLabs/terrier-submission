// Checks for external prerequisites needed by the plagiarism checker

use std::error::Error;
use std::io::ErrorKind;
use std::process::{Command, Stdio};

/// Verify that the required external tool `copydetect` is available on PATH.
///
/// Returns Ok(()) if the command can be spawned. If the executable is not found,
/// returns an Error with guidance on how to install or expose it on PATH.
pub fn check_prereq() -> Result<(), Box<dyn Error>> {
    let try_spawn = Command::new("copydetect")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match try_spawn {
        Ok(_status) => Ok(()),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Err("Missing required tool 'copydetect'. Please install it and ensure it is on your PATH.\n\
                     Try one of the following:\n\
                       - pipx install copydetect\n\
                       - pip install --user copydetect   # then ensure ~/.local/bin is on PATH\n\
                       - If using Homebrew Python: pipx ensurepath && pipx install copydetect".to_string()
                .into())
            } else {
                Err(format!("Failed to check for 'copydetect': {}", e).into())
            }
        }
    }
}
