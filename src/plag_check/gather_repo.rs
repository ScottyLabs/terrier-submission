use crate::git_tools::repository::GithubRepo;
use octocrab::Octocrab;
use std::path::PathBuf;

async fn gather_repo_urls_and_sizes_from_user(
    octocrab: &Octocrab,
    username: &str,
) -> octocrab::Result<Vec<(String, u32)>> {
    let repos_page = octocrab.users(username).repos().per_page(20).send().await?;

    let mut res = Vec::<(String, u32)>::new();

    for repo in repos_page {
        let url = repo.html_url.as_ref().map(|u| u.as_str().to_string());
        let size_kb = repo.size.unwrap_or(0);
        if let Some(url) = url {
            res.push((url, size_kb));
        }
    }

    Ok(res)
}

pub async fn gather_repo_urls_from_user(
    octocrab: &Octocrab,
    username: &str,
) -> octocrab::Result<Vec<String>> {
    let repos_with_sizes = gather_repo_urls_and_sizes_from_user(octocrab, username).await?;
    Ok(repos_with_sizes.into_iter().map(|(url, _)| url).collect())
}

async fn clone_repos_into_dir_with_size_limit(
    repo_urls: Vec<(String, u32)>,
    target_dir: &PathBuf,
    size_threshold_kb: u32,
) -> Result<Vec<GithubRepo>, git2::Error> {
    let mut res = Vec::<GithubRepo>::new();
    let mut total_cumulative_size: u32 = 0;
    for (url, size) in repo_urls {
        if total_cumulative_size + size < size_threshold_kb {
            total_cumulative_size += size;
            let local_path = target_dir.join(&url);
            let repo = GithubRepo::new_with_local_path(&*url, local_path.to_str().unwrap())?;
            res.push(repo);
        }
    }

    Ok(res)
}

pub async fn clone_repos_into_dir(
    repo_urls: &Vec<String>,
    target_dir: PathBuf,
) -> Result<Vec<GithubRepo>, git2::Error> {
    let mut res = Vec::<GithubRepo>::new();
    for url in repo_urls {
        let local_path = target_dir.join(url);
        let repo = GithubRepo::new_with_local_path(url, local_path.to_str().unwrap())?;
        res.push(repo);
    }

    Ok(res)
}
