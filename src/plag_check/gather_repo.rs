use git2::Repository;
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
    repo_urls: Vec<(String, String)>,
    target_dir: &PathBuf,
) -> Result<(), git2::Error> {
    for (url, branch) in repo_urls {
        let local_path = target_dir.join(&url);
        let repo = Repository::clone(&*url, &local_path)?;
    }

    Ok(())
}
