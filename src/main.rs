mod git_tools;
mod plag_check;

use crate::git_tools::verification::VerificationResult::Failed;
use crate::plag_check::copydetect::{CopydetectError, run_copydetect};
use crate::plag_check::gather_repo::{clone_repos_into_dir, gather_repo_urls_and_sizes_from_user};
use crate::plag_check::plag_result::{PlagiarismVerificationResult, copy_percentage_from_html};
use crate::plag_check::prereq_check::check_prereq;
use crate::plag_check::verification::VerificationResult;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering::{Greater, Less};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    repo: String,
    usernames: Vec<String>,
    start_time: u64,
    end_time: u64,
    #[serde(default = "default_size_threshold")]
    size_threshold_kb: u32,
    #[serde(default = "default_display_threshold")]
    display_threshold: f32,
}

fn default_size_threshold() -> u32 {
    100_000 // ~100MB default
}

fn default_display_threshold() -> f32 {
    0.33 //Default 33% similarity
}

#[derive(Debug, Serialize)]
struct VerificationOutput {
    metadata: git_tools::metadata::MetadataVerificationResult,
    plagiarism: PlagiarismVerificationResult,
    github_issues: Vec<String>,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, required = true)]
    path: String,

    #[arg(short, long, required = true)]
    mode: String,
}

fn sort_verification_result(mut output: Vec<VerificationOutput>) -> Vec<VerificationOutput> {
    output.sort_by(|a, b| {
        let ameta = &a.metadata;
        let bmeta = &b.metadata;

        match (&ameta.first_commit_time, &bmeta.first_commit_time) {
            (Failed(_), _) => return Less,
            (_, Failed(_)) => return Greater,
            _ => {}
        }

        match (&ameta.contributors, &bmeta.contributors) {
            (Failed(_), _) => return Less,
            (_, Failed(_)) => return Greater,
            _ => {}
        }

        match (&a.plagiarism.result, &b.plagiarism.result) {
            (VerificationResult::ManualRequired, _) => return Less,
            (_, VerificationResult::ManualRequired) => return Greater,
            _ => {}
        }

        let a_length = &a.github_issues.len();
        let b_length = &b.github_issues.len();

        if a_length > b_length {
            return Less;
        } else if a_length < b_length {
            return Greater;
        }

        let a_percent = &a.plagiarism.result.get_percent();
        let b_percent = &b.plagiarism.result.get_percent();

        if a_percent >= b_percent {
            return Less;
        } else {
            return Greater;
        }
    });

    output
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
    let contents = fs::read_to_string(path).map_err(|_e| {
        format!(
            "The JSON file provided ('{}') does not exist or could not be read.",
            path
        )
    })?;

    let data: ConfigData = serde_json::from_str(&contents).map_err(|err| {
        format!(
            "The JSON file provided ('{}') is not a valid JSON file: {}",
            path, err
        )
    })?;

    Ok(data)
}

fn load_multiple(path: &str) -> Result<Vec<ConfigData>, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path).map_err(|_e| {
        format!(
            "The JSON file provided ('{}') does not exist or could not be read.",
            path
        )
    })?;

    let data: Vec<ConfigData> = serde_json::from_str(&contents).map_err(|err| {
        format!(
            "The JSON file provided ('{}') is not a valid JSON file: {}",
            path, err
        )
    })?;

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

fn metadata_result_from_clone_error(
    err: git2::Error,
) -> git_tools::metadata::MetadataVerificationResult {
    use crate::git_tools::verification::{FailureReason, VerificationResult};

    git_tools::metadata::MetadataVerificationResult::new(
        VerificationResult::Failed(FailureReason::GitError(err)),
        VerificationResult::Failed(FailureReason::GitError(git2::Error::from_str(
            "main repository clone failed (see first error)",
        ))),
        VerificationResult::Failed(FailureReason::GitError(git2::Error::from_str(
            "main repository clone failed (see first error)",
        ))),
    )
}

async fn collect_user_repos(
    octocrab: &octocrab::Octocrab,
    usernames: &[String],
    main_repo: &str,
    copydetect_path: &Path,
    size_threshold_kb: u32,
    start_time: u64,
    clone_repos: bool,
    github_issues: &mut Vec<String>,
) -> Vec<git_tools::repository::GithubRepo> {
    let mut repo_infos = vec![];
    for user in usernames {
        let urls_with_sizes = match gather_repo_urls_and_sizes_from_user(octocrab, user, start_time, main_repo.into())
            .await
        {
            Ok(urls) => urls,
            Err(err) => {
                github_issues.push(format!("Failed to list repos for user '{}': {}", user, err));
                continue;
            }
        };

        repo_infos.push(urls_with_sizes);
    }

    let mut all_repos = vec![];
    if clone_repos {
        let mut to_clone = vec![];

        let mut cont = true;
        let mut i = 0;
        while cont {
            cont = false;
            for info in &repo_infos {
                if i >= info.len() {
                    continue;
                }
                cont = true;
                to_clone.push(info[i].clone());
            }
            i += 1;
        }

        let repos =
            clone_repos_into_dir(to_clone, copydetect_path, size_threshold_kb, github_issues).await;
        all_repos.extend(repos);
    }
    all_repos
}

fn run_plagiarism_check(
    main_repo_path: &str,
    comparison_repos: &[git_tools::repository::GithubRepo],
    display_threshold: f32,
    working_dir: &Path,
) -> PlagiarismVerificationResult {
    let comparison_paths: Vec<&str> = comparison_repos
        .iter()
        .map(|repo| repo.local_path.as_str())
        .collect();

    match run_copydetect(
        &[main_repo_path],
        &comparison_paths,
        display_threshold,
        working_dir,
    ) {
        Ok(Some(report_path)) => {
            let plag_score = copy_percentage_from_html(&report_path);
            PlagiarismVerificationResult::new(plag_score, Some(report_path))
        }
        Ok(None) => {
            eprintln!("copydetect skipped because no comparison repositories were available.");
            PlagiarismVerificationResult::manual(None)
        }
        Err(err) => {
            eprintln!("copydetect failed: {}", err);
            let report_path = match err {
                CopydetectError::MissingReport(path) => Some(path),
                _ => None,
            };
            PlagiarismVerificationResult::manual(report_path)
        }
    }
}

fn save_results(
    verification_output: &VerificationOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(verification_output)?;
    let output_dir = Path::new("output");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("result.json"), serialized)?;

    if let Some(report_path) = &verification_output.plagiarism.report_path {
        if report_path.exists() {
            fs::copy(report_path, output_dir.join("report.html"))?;
        } else {
            eprintln!(
                "copydetect report was expected at {}, but the file does not exist",
                report_path.display()
            );
        }
    }

    Ok(())
}

fn save_all(
    verification_output: Vec<VerificationOutput>,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(&verification_output)?;
    let output_dir = Path::new("output");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("result.json"), serialized)?;

    Ok(())
}

fn setup_copydetect_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn cleanup_repos(repos: Vec<git_tools::repository::GithubRepo>, copydetect_path: &Path) {
    for repo in repos {
        repo.destroy();
    }
    let _ = fs::remove_dir_all(copydetect_path);
}

async fn single_input(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let data = load_config(&args.path)?;

    let repo_constraints = build_metadata_constraints(&data);
    let mut github_issues = Vec::new();
    let (github_repo, metadata_result) =
        match git_tools::repository::GithubRepo::new(&data.repo, false) {
            Ok(repo) => {
                let metadata_result =
                    git_tools::metadata::check_metadata_at_path(&repo.local_path, repo_constraints);
                (Some(repo), metadata_result)
            }
            Err(err) => {
                github_issues.push(format!(
                    "Failed to clone main repo '{}': {}",
                    data.repo, err
                ));
                (None, metadata_result_from_clone_error(err))
            }
        };

    let copydetect_path = PathBuf::from("/tmp/repo_copydetect");
    setup_copydetect_dir(&copydetect_path)?;

    let octocrab = octocrab::Octocrab::builder().build()?;
    let all_repos = collect_user_repos(
        &octocrab,
        &data.usernames,
        &data.repo,
        &copydetect_path,
        data.size_threshold_kb,
        data.start_time,
        github_repo.is_some(),
        &mut github_issues,
    )
    .await;

    let plagiarism_result = match &github_repo {
        Some(repo) => run_plagiarism_check(
            &repo.local_path,
            &all_repos,
            data.display_threshold,
            &copydetect_path,
        ),
        None => PlagiarismVerificationResult::manual(None),
    };

    let verification_output = VerificationOutput {
        metadata: metadata_result,
        plagiarism: plagiarism_result,
        github_issues,
    };

    save_results(&verification_output)?;

    cleanup_repos(all_repos, &copydetect_path);
    if let Some(repo) = github_repo {
        repo.destroy();
    }

    Ok(())
}

async fn multiple_inputs(
    data: ConfigData,
) -> Result<VerificationOutput, Box<dyn std::error::Error>> {
    let repo_constraints = build_metadata_constraints(&data);
    let mut github_issues = Vec::new();
    let (github_repo, metadata_result) =
        match git_tools::repository::GithubRepo::new(&data.repo, false) {
            Ok(repo) => {
                let metadata_result =
                    git_tools::metadata::check_metadata_at_path(&repo.local_path, repo_constraints);
                (Some(repo), metadata_result)
            }
            Err(err) => {
                github_issues.push(format!(
                    "Failed to clone main repo '{}': {}",
                    data.repo, err
                ));
                (None, metadata_result_from_clone_error(err))
            }
        };

    let copydetect_path = PathBuf::from("/tmp/repo_copydetect");
    setup_copydetect_dir(&copydetect_path)?;

    let octocrab = octocrab::Octocrab::builder().build()?;
    let all_repos = collect_user_repos(
        &octocrab,
        &data.usernames,
        &data.repo,
        &copydetect_path,
        data.size_threshold_kb,
        data.start_time,
        github_repo.is_some(),
        &mut github_issues,
    )
    .await;

    // TODO: Check if repo is empty and return empty repo result if so
    let plagiarism_result = match &github_repo {
        Some(repo) => run_plagiarism_check(
            &repo.local_path,
            &all_repos,
            data.display_threshold,
            &copydetect_path,
        ),
        None => PlagiarismVerificationResult::manual(None),
    };

    let verification_output = VerificationOutput {
        metadata: metadata_result,
        plagiarism: plagiarism_result,
        github_issues,
    };

    cleanup_repos(all_repos, &copydetect_path);
    if let Some(repo) = github_repo {
        repo.destroy();
    }

    Ok(verification_output)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    verify_prerequisites()?;

    if args.mode == "single" {
        single_input(&args);
    } else if args.mode == "all" {
        let mut outputs: Vec<VerificationOutput> = Vec::new();
        let data = load_multiple(&args.path)?;
        for point in data {
            println!("");
            outputs.push(multiple_inputs(point).await?);
        }

        outputs = sort_verification_result(outputs);

        save_all(outputs)?;
    }

    Ok(())
}
