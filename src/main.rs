mod file_tools;
mod git_tools;
mod zip_tools;
mod plag_check;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    zip: PathBuf,
    repo: String,
    branch: String,
    usernames: Vec<String>,
    start_time: u64,
    end_time: u64,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, required = true)]
    path: String,
}

fn system_time_from_unix_secs(secs: u64) -> std::time::SystemTime {
    std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(&args.path)
        .map_err(|_e| format!("The JSON file provided ('{}') does not exist.", &args.path))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: ConfigData = serde_json::from_str(&contents).expect(&format!(
        "The JSON file provided ('{}') is not a valid JSON file",
        &args.path
    ));

    println!("Input Data:\n{:?}", &data);
    println!("\n----------------\n");

    let github_repo = git_tools::repository::GithubRepo::new(&data.repo, &data.branch)?;

    let repo_constraints = git_tools::metadata::MetadataConstraints {
        first_commit_time: Some(
            system_time_from_unix_secs(data.start_time)..system_time_from_unix_secs(data.end_time),
        ),
        last_commit_time: Some(
            system_time_from_unix_secs(data.start_time)..system_time_from_unix_secs(data.end_time),
        ),
        usernames: Some(data.usernames),
    };
    let repo_check_res =
        git_tools::metadata::check_metadata_at_path(&github_repo.local_path, repo_constraints);

    github_repo.destroy();

    println!("Result Data:\n{:?}", repo_check_res);

    let serialized = serde_json::to_string_pretty(&repo_check_res)?;
    let mut output = File::create("result.json")?;
    output.write_all(serialized.as_bytes())?;

    Ok(())
}
