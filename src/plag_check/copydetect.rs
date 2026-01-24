use std::process::Command;
use std::fs;

/// Runs copydetect
pub fn run_copydetect(test_dirs: Vec<&str>, ref_dirs: Vec<&str>, display_threshold: f32)  -> Result<&'static str, Box<dyn std::error::Error>> {
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
        "zig",   // Zig
        "scala", // Scala
        "clj",   // Clojure
        "cljs",  // ClojureScript
        "groovy",// Groovy
        "vb",    // Visual Basic
        "erl",   // Erlang
        "ex",    // Elixir
        "exs",   // Elixir Script

        // Systems Programming
        "c",     // C
        "cpp",   // C++
        "h",     // C/C++ Header
        "hpp",   // C++ Header
        "s",     // Assembly
        "asm",   // Assembly (alt)
        "ml",    // OCaml / Standard ML
        "mli",   // OCaml Interface
        "nim",   // Nim
        "d",     // D Language

        // Web Frontend
        "html",  // HTML
        "css",   // CSS
        "scss",  // SASS/SCSS
        "jsx",   // JavaScript XML (React)
        "tsx",   // TypeScript XML (React)
        "vue",   // Vue SFC
        "svelte",// Svelte
        "astro", // Astro
        "md",    // Markdown
        "mdx",   // Markdown + JSX

        // Scripting & Data
        "sh",    // Shell Script
        "ps1",   // PowerShell
        "sql",   // SQL
        "r",     // R
        "pl",    // Perl
        "lua",   // Lua
        "pyw",   // Python (no console)
        "bat",   // Windows Batch
        "cmd",   // Windows Command Script
        "awk",   // AWK
        "tcl",   // Tcl
        "m",     // MATLAB / Octave / Objective‑C (context‑dependent)
        "jl",    // Julia
        "yaml",  // YAML
        "yml",   // YAML (alt)
        "toml",  // TOML
        "ini",   // INI Config

        // Functional Languages
        "hs",       // Haskell
        "lhs",      // Literate Haskell
        "fs",       // F#
        "fsi",      // F# Script
        "fsscript", // F# Script (alt)

        // Mobile / Cross‑Platform
        "mm",    // Objective‑C++
        // "m" already included above

        // Infra / Build / DevOps
        "gradle", // Gradle Build
        "mk",     // Makefile (alt)
        "dockerfile", // Dockerfile
        "tf",     // Terraform
        "tfvars", // Terraform Variables
    ];

    if ref_dirs.len() == 0 {
        return Ok("Manual")
    }

    let mut try_spawn = Command::new("copydetect")
        .arg("-t")
        .args(test_dirs)
        .arg("-r")
        .args(ref_dirs)
        .arg("-e")
        .args(extensions)
        .arg("-d")
        .args(vec![format!("{}", display_threshold)])
        .arg("-a")
        .status();
    if let Ok(status) = try_spawn {
        return Ok("Passed")
    } else {
        return Ok("Manual")
    }
}
