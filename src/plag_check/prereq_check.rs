use std::process::{Command, Stdio};

pub fn check_prereq() -> bool {
    Command::new("copydetect")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
