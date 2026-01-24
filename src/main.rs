mod file_tools;
mod git_tools;
mod plag_check;
mod zip_tools;

use crate::plag_check::copydetect::run_copydetect;
use crate::plag_check::gather_repo::{clone_repos_into_dir, gather_repo_urls_and_sizes_from_user};
use crate::plag_check::plag_result::{PlagiarismVerificationResult, copy_percentage_from_html};
use crate::plag_check::prereq_check::check_prereq;
use crate::plag_check::verification::VerificationResult::{Verified, ManualRequired};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    repo: String,
    usernames: Vec<String>,
    start_time: u64,
    end_time: u64,
    #[serde(default = "default_size_threshold")]
    size_threshold_kb: u32,
    #[serde(default = "default_display_threshold")]
    display_threshold: f32
}

fn default_size_threshold() -> u32 {
    100_000 // ~100MB default
}

fn default_display_threshold() -> f32 {
    0.33 //Default 33% similarity
}

/// Combined verification result containing both metadata and plagiarism checks
#[derive(Debug, Serialize)]
struct VerificationOutput {
    /// Metadata verification results (commit times, contributors)
    metadata: git_tools::metadata::MetadataVerificationResult,
    /// Plagiarism verification result (similarity percentage)
    plagiarism: PlagiarismVerificationResult,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, required = true)]
    path: String,
}

fn system_time_from_unix_secs(secs: u64) -> std::time::SystemTime {
    std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs)
}

fn verify_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
    if !check_prereq() {
        println!(
            "Missing required tool 'copydetect'. Please install it and ensure it is on your PATH.\n\
             Try one of the following:\n\
               - pipx install copydetect\n\
               - pip install copydetect\n\
               - uv tool install copydetect"
        );
        return Err("Missing required tool 'copydetect'.".into());
    }
    Ok(())
}

fn load_config(path: &str) -> Result<ConfigData, Box<dyn std::error::Error>> {
    let mut file = File::open(path)
        .map_err(|_e| format!("The JSON file provided ('{}') does not exist.", path))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: ConfigData = serde_json::from_str(&contents).expect(&format!(
        "The JSON file provided ('{}') is not a valid JSON file",
        path
    ));

    Ok(data)
}

fn build_metadata_constraints(data: &ConfigData) -> git_tools::metadata::MetadataConstraints {
    git_tools::metadata::MetadataConstraints {
        first_commit_time: Some(
            system_time_from_unix_secs(data.start_time)..system_time_from_unix_secs(data.end_time),
        ),
        last_commit_time: Some(
            system_time_from_unix_secs(data.start_time)..system_time_from_unix_secs(data.end_time),
        ),
        usernames: Some(data.usernames.clone()),
    }
}

async fn collect_user_repos(
    octocrab: &octocrab::Octocrab,
    usernames: &[String],
    main_repo: &str,
    copydetect_path: &PathBuf,
    size_threshold_kb: u32,
    start_time: u64
) -> Result<Vec<git_tools::repository::GithubRepo>, Box<dyn std::error::Error>> {
    let mut all_repos = vec![];
    for user in usernames {
        let urls_with_sizes = gather_repo_urls_and_sizes_from_user(octocrab, user, start_time)
            .await?
            .into_iter()
            .filter(|(url, _)| *url != main_repo)
            .collect::<Vec<_>>();
        let repos =
            clone_repos_into_dir(urls_with_sizes, copydetect_path, size_threshold_kb).await?;
        all_repos.extend(repos);
    }
    Ok(all_repos)
}

fn run_plagiarism_check(
    main_repo_path: &str,
    comparison_repos: &[git_tools::repository::GithubRepo],
    display_threshold: f32
) -> PlagiarismVerificationResult {
    let comparison_paths: Vec<&str> = comparison_repos
        .iter()
        .map(|repo| &*repo.local_path)
        .collect();

    let res = run_copydetect(vec![main_repo_path], comparison_paths, display_threshold);
    if res.unwrap() == "Passed" {
        let plag_score = copy_percentage_from_html(Some("report.html".parse().unwrap()));
        PlagiarismVerificationResult::new(plag_score)
    } else {
        PlagiarismVerificationResult::new(None)
    }
}

fn save_results(
    verification_output: &VerificationOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(verification_output)?;
    let mut output = File::create("result.json")?;
    output.write_all(serialized.as_bytes())?;

    std::fs::remove_dir_all("output").ok();
    std::fs::create_dir("output")?;

    println!("Reached!");

    match &verification_output.plagiarism.result {
        Verified(percent) => {
            std::fs::rename("report.html", "output/report.html")?;
        }
        _ => {}
    }

    std::fs::rename("result.json", "output/result.json")?;

    Ok(())
}

fn setup_copydetect_dir(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::remove_dir_all(path).ok();
    std::fs::create_dir(path)?;
    Ok(())
}

fn cleanup_repos(repos: Vec<git_tools::repository::GithubRepo>, copydetect_path: &PathBuf) {
    for repo in repos {
        repo.destroy();
    }
    std::fs::remove_dir_all(copydetect_path).ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    verify_prerequisites()?;

    let args = Args::parse();
    let data = load_config(&args.path)?;

    println!("Input Data:\n{:?}", &data);
    println!("\n----------------\n");

    let github_repo = git_tools::repository::GithubRepo::new(&data.repo, false)?;
    let repo_constraints = build_metadata_constraints(&data);
    let metadata_result =
        git_tools::metadata::check_metadata_at_path(&github_repo.local_path, repo_constraints);

    let copydetect_path = PathBuf::from("/tmp/repo_copydetect");
    setup_copydetect_dir(&copydetect_path)?;

    let octocrab = octocrab::Octocrab::builder().build()?;
    let all_repos = collect_user_repos(
        &octocrab,
        &data.usernames,
        &data.repo,
        &copydetect_path,
        data.size_threshold_kb,
        data.start_time
    )
    .await?;

    //TODO: Check if repo is empty and return empty repo result if so
    let plagiarism_result = run_plagiarism_check(&github_repo.local_path, &all_repos, data.display_threshold);

    cleanup_repos(all_repos, &copydetect_path);
    github_repo.destroy();

    let verification_output = VerificationOutput {
        metadata: metadata_result,
        plagiarism: plagiarism_result,
    };

    println!("Result Data:\n{:?}", verification_output);

    save_results(&verification_output)?;

    Ok(())
}
