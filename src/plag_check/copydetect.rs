use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{error, io};

const DEFAULT_EXTENSIONS: &[&str] = &[
    // General Purpose & Web Backend
    "py",
    "js",
    "ts",
    "java",
    "cs",
    "go",
    "rs",
    "rb",
    "php",
    "kt",
    "swift",
    "dart",
    "zig",
    "scala",
    "clj",
    "cljs",
    "groovy",
    "vb",
    "erl",
    "ex",
    "exs", // Systems Programming
    "c",
    "cpp",
    "h",
    "hpp",
    "s",
    "asm",
    "ml",
    "mli",
    "nim",
    "d", // Web Frontend
    "html",
    "css",
    "scss",
    "jsx",
    "tsx",
    "vue",
    "svelte",
    "astro",
    "md",
    "mdx", // Scripting & Data
    "sh",
    "ps1",
    "sql",
    "r",
    "pl",
    "lua",
    "pyw",
    "bat",
    "cmd",
    "awk",
    "tcl",
    "m",
    "jl",
    "yaml",
    "yml",
    "toml",
    "ini", // Functional Languages
    "hs",
    "lhs",
    "fs",
    "fsi",
    "fsscript", // Mobile / Cross‑Platform
    "mm",       // Infra / Build / DevOps
    "gradle",
    "mk",
    "dockerfile",
    "tf",
    "tfvars",
];

#[derive(Debug)]
pub enum CopydetectError {
    Spawn(io::Error),
    NonZeroExit(Option<i32>),
    MissingReport(PathBuf),
}

impl fmt::Display for CopydetectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopydetectError::Spawn(err) => write!(f, "failed to spawn copydetect: {}", err),
            CopydetectError::NonZeroExit(code) => {
                write!(f, "copydetect exited with status {:?}", code)
            }
            CopydetectError::MissingReport(path) => {
                write!(
                    f,
                    "copydetect did not produce a report at {}",
                    path.display()
                )
            }
        }
    }
}

impl error::Error for CopydetectError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CopydetectError::Spawn(err) => Some(err),
            _ => None,
        }
    }
}

/// Runs copydetect and returns the path to the generated report, if any.
pub fn run_copydetect(
    test_dirs: &[&str],
    ref_dirs: &[&str],
    display_threshold: f32,
    working_dir: &Path,
) -> Result<Option<PathBuf>, CopydetectError> {
    if ref_dirs.is_empty() {
        return Ok(None);
    }

    let status = Command::new("copydetect")
        .current_dir(working_dir)
        .arg("-t")
        .args(test_dirs)
        .arg("-r")
        .args(ref_dirs)
        .arg("-e")
        .args(DEFAULT_EXTENSIONS)
        .arg("-d")
        .arg(display_threshold.to_string())
        .arg("-a")
        .status()
        .map_err(CopydetectError::Spawn)?;

    if !status.success() {
        return Err(CopydetectError::NonZeroExit(status.code()));
    }

    let report_path = working_dir.join("report.html");
    if report_path.exists() {
        Ok(Some(report_path))
    } else {
        Err(CopydetectError::MissingReport(report_path))
    }
}
