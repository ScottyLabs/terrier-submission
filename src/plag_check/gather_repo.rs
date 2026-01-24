use crate::git_tools::repository::GithubRepo;
use chrono::{DateTime, TimeZone, Utc};
use octocrab::{Octocrab, params};
use std::path::PathBuf;
use rand::{thread_rng, Rng};
use std::fs;

pub async fn gather_repo_urls_and_sizes_from_user(
    octocrab: &Octocrab,
    username: &str,
    start_time: u64 
) -> octocrab::Result<Vec<(String, u32)>> {
    let repos_page = octocrab
        .users(username)
        .repos()
        .sort(params::repos::Sort::Updated)
        .direction(params::Direction::Descending)
        .per_page(50)
        .send()
        .await?;

    let mut res = Vec::<(String, u32)>::new();
    let cutoff_dt = Some(Utc.timestamp_opt(start_time as i64, 0).unwrap());

    for repo in repos_page {
        let url = repo.html_url.as_ref().map(|u| u.as_str().to_string());
        let size_kb = repo.size.unwrap_or(0);
        if let Some(url) = url {
            if let (Some(created), Some(cutoff)) = (repo.created_at, cutoff_dt) {
                if (created < cutoff) {
                    res.push((url, size_kb));
                }
            }
        }
    }

    println!("\nREPOS:\n");
    for (x, y) in &res {
        println!("{}", format!("{}, {}\n", x, y));
    }
    println!("\n");

    Ok(res)
}

fn random_string(len: u32) -> String {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = thread_rng();

    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..LETTERS.len());
            LETTERS[idx] as char
        })
        .collect()
}

pub fn is_dir_empty(path: &std::path::Path) -> std::io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

pub async fn clone_repos_into_dir(
    repo_urls: Vec<(String, u32)>,
    target_dir: &PathBuf,
    size_threshold_kb: u32,
) -> Result<Vec<GithubRepo>, git2::Error> {
    let mut res = Vec::<GithubRepo>::new();
    let mut total_cumulative_size: u32 = 0;
    let mut clone_counter: u32 = 0;
    for (url, size) in repo_urls {
        if total_cumulative_size + size < size_threshold_kb {
            total_cumulative_size += size;
            let local_path = target_dir.join(random_string(50));
            clone_counter += 1;
            let repo = GithubRepo::new_with_local_path(&*url, local_path.to_str().unwrap(), true)?;
            if !is_dir_empty(&local_path).unwrap_or(true) {
                res.push(repo);
            }
        }
    }

    Ok(res)
}
