use std::process::Command;

/// Runs copydetect
pub fn run_copydetect(test_dirs: Vec<&str>, ref_dirs: Vec<&str>) {
    let extensions = vec![
        // General Purpose & Web Backend
        "py",    // Python
        "js",    // JavaScript
        "ts",    // TypeScript
        "java",  // Java
        "cs",    // C#
        "go",    // Go
        "rs",    // Rust
        "rb",    // Ruby
        "php",   // PHP
        "kt",    // Kotlin
        "swift", // Swift
        "dart",  // Dart
        "zig",  // Zig
        // Systems Programming
        "c",   // C
        "cpp", // C++
        "h",   // C/C++ Header
        "hpp", // C++ Header
        // Web Frontend
        "html", // HTML
        "css",  // CSS
        "scss", // SASS/SCSS
        "jsx",  // JavaScript XML (React)
        "tsx",  // TypeScript XML (React)
        // Scripting & Data
        "sh",  // Shell Script
        "ps1", // PowerShell
        "sql", // SQL
        "r",   // R
        "pl",  // Perl
        "lua", // Lua
    ];
    let mut try_spawn = Command::new("copydetect")
        .arg("-t")
        .args(test_dirs)
        .arg("-r")
        .args(ref_dirs)
        .arg("-e")
        .args(extensions)
        .arg("-a")
        .status();
    if let Ok(status) = try_spawn {
        // TODO: figure out what each status means
        // I'm gonna assume that report.html is created
    } else {
        panic!("Failed to spawn copydetect:\n {}", try_spawn.unwrap_err());
    }
}
