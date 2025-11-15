use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository, Sort};
use std::time::{Duration, SystemTime};

/// Get the repository creation time, defined as the timestamp of the oldest commit reachable
/// from HEAD (or from any reference if HEAD is unavailable). Falls back to UNIX_EPOCH
/// if no commits can be found.
pub fn get_creation_time(repo: &Repository) -> SystemTime {
    let mut walk = match repo.revwalk() {
        Ok(w) => w,
        Err(_) => return SystemTime::UNIX_EPOCH,
    };

    // Prefer walking from HEAD; if that fails (e.g., empty repo), try all references.
    if walk.push_head().is_err() {
        if let Ok(refs) = repo.references() {
            for r in refs.flatten() {
                if let Some(oid) = r.target() {
                    let _ = walk.push(oid);
                }
            }
        }
    }

    // Sort by commit time to make finding the earliest trivial.
    let _ = walk.set_sorting(Sort::TIME);

    let mut earliest: Option<i64> = None;

    for oid_result in walk {
        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let ts = commit.time().seconds();
        earliest = Some(match earliest {
            Some(e) => e.min(ts),
            None => ts,
        });
    }

    match earliest {
        Some(secs) if secs >= 0 => SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64),
        Some(secs) => SystemTime::UNIX_EPOCH - Duration::from_secs((-secs) as u64),
        None => SystemTime::UNIX_EPOCH,
    }
}

pub struct GithubRepo {
    pub url: String,
    pub local_path: String,
    pub repo: Repository,
}

impl GithubRepo {
    pub fn new(link: &str) -> Result<Self, git2::Error> {
        let local_path = format!("/tmp/repo_{}", uuid::Uuid::new_v4());
        Self::new_with_local_path(link, &local_path)
    }

    pub fn new_with_local_path(link: &str, local_path: &str) -> Result<Self, git2::Error> {
        eprintln!("Creating new Github repo at {}", &local_path);
        eprintln!("Cloning: {}", link);
        let mut opt = FetchOptions::new();
        opt.depth(1);
        let repo = RepoBuilder::new()
            .fetch_options(opt)
            .clone(link, (&local_path).as_ref())?;
        Ok(Self {
            url: link.to_string(),
            local_path: local_path.to_string(),
            repo,
        })
    }

    pub fn get_creation_time(&self) -> SystemTime {
        get_creation_time(&self.repo)
    }

    pub fn destroy(self) {
        let _ = std::fs::remove_dir_all(&self.local_path);
    }
}

#[cfg(test)]
mod tests {
    use super::get_creation_time;
    use git2::{Repository, Signature};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    fn unique_temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("git_creation_time_test_{}", uuid::Uuid::new_v4()));
        path
    }

    fn write_file<P: AsRef<Path>>(root: P, rel: &str, contents: &str) {
        let full = root.as_ref().join(rel);
        if let Some(parent) = full.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut f = File::create(full).expect("create file");
        f.write_all(contents.as_bytes()).expect("write file");
    }

    fn commit_all(repo: &Repository, message: &str) {
        // Add all changes
        let mut index = repo.index().expect("open index");
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .expect("add all");
        index.write().expect("index write");
        let tree_id = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_id).expect("find tree");

        let sig = Signature::now("Tester", "tester@example.com").expect("sig now");
        let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        match parent_commit {
            Some(parent) => {
                let _ = repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
                    .expect("commit with parent");
            }
            None => {
                let _ = repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                    .expect("initial commit");
            }
        }
    }

    #[test]
    fn test_empty_repo_returns_unix_epoch() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let repo = Repository::init(&dir).expect("init repo");
        let t = get_creation_time(&repo);
        assert_eq!(t, SystemTime::UNIX_EPOCH);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_repo_two_commits_returns_earliest_time() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let repo = Repository::init(&dir).expect("init repo");

        // First commit
        write_file(&dir, "a.txt", "one");
        commit_all(&repo, "first");
        let first_oid = repo.head().unwrap().peel_to_commit().unwrap().id();
        let first_time = repo.find_commit(first_oid).unwrap().time().seconds();

        // Ensure the next commit has a distinct later timestamp
        std::thread::sleep(Duration::from_millis(67));

        // Second commit
        write_file(&dir, "a.txt", "two");
        commit_all(&repo, "second");
        let second_oid = repo.head().unwrap().peel_to_commit().unwrap().id();
        let second_time = repo.find_commit(second_oid).unwrap().time().seconds();

        assert!(second_time >= first_time);

        let creation = get_creation_time(&repo);
        let expected = if first_time <= second_time {
            first_time
        } else {
            second_time
        };

        let expected_time = if expected >= 0 {
            SystemTime::UNIX_EPOCH + Duration::from_secs(expected as u64)
        } else {
            SystemTime::UNIX_EPOCH - Duration::from_secs((-expected) as u64)
        };

        assert_eq!(creation, expected_time);

        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod github_tests {
    use super::GithubRepo;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_github_repo_creation_time() {
        let repo_url = "https://github.com/ScottyLabs/terrier-submission";
        let github_repo = GithubRepo::new(repo_url).expect("clone repo");
        let creation_time = github_repo.get_creation_time();
        // Sep 20 2025 4:28 PM EDT
        // 1758400104
        let expected_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1758400104);
        assert_eq!(creation_time, expected_time);
        github_repo.destroy();
    }

    #[test]
    fn test_github_repo_invalid_url() {
        let repo_url = "https://github.com/ScottyLabs/terrier-submission-bad-url";
        let result = GithubRepo::new(repo_url);
        assert!(result.is_err());
    }
}
