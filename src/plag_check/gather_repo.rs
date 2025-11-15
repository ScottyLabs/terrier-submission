use crate::git_tools::repository::GithubRepo;
use octocrab::Octocrab;
use std::path::PathBuf;

async fn gather_repo_urls_from_user(
    octocrab: Octocrab,
    username: &str,
) -> octocrab::Result<Vec<String>> {
    let repos_page = octocrab.users(username).repos().per_page(20).send().await?;

    let mut res = Vec::<String>::new();

    for repo in repos_page {
        let url = repo.html_url.as_ref().map(|u| u.as_str().to_string());
        if let Some(url) = url {
            res.push(url);
        }
    }

    Ok(res)
}

async fn clone_repos_into_dir(
    repo_urls: Vec<String>,
    target_dir: &PathBuf,
) -> Result<Vec<GithubRepo>, git2::Error> {
    let mut res = Vec::<GithubRepo>::new();
    for url in repo_urls {
        let local_path = target_dir.join(&url);
        let repo = GithubRepo::new_with_local_path(&*url, local_path.to_str().unwrap())?;
        res.push(repo);
    }

    Ok(res)
}
