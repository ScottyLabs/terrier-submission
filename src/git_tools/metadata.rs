use crate::git_tools::verification::{FailureReason, VerificationResult};
use git2::{Repository, Sort, Time as GitTime};
use std::ops::Range;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Constraints for a Git repository's timeline.
/// In each constraint, `None` means no constraint is applied.
#[derive(Debug, Clone)]
pub struct MetadataConstraints {
    /// Range of allowed first (earliest) commit time
    pub first_commit_time: Option<Range<SystemTime>>,
    /// Range of allowed last (latest) commit time
    pub last_commit_time: Option<Range<SystemTime>>,
}

impl MetadataConstraints {
    /// Create an empty set of constraints (everything skipped)
    pub fn new_empty() -> Self {
        Self {
            first_commit_time: None,
            last_commit_time: None,
        }
    }

    /// Create a new `MetadataConstraints`
    pub fn new(
        first_commit_time: Option<Range<SystemTime>>,
        last_commit_time: Option<Range<SystemTime>>,
    ) -> Self {
        Self {
            first_commit_time,
            last_commit_time,
        }
    }
}

/// The result of verifying a repository's metadata against a set of constraints.
#[derive(Debug)]
pub struct MetadataVerificationResult {
    /// Result of verifying the first (earliest) commit time
    pub first_commit_time: VerificationResult,
    /// Result of verifying the last (latest) commit time
    pub last_commit_time: VerificationResult,
}

impl MetadataVerificationResult {
    /// Create a new `MetadataVerificationResult`
    pub fn new(first: VerificationResult, last: VerificationResult) -> Self {
        Self {
            first_commit_time: first,
            last_commit_time: last,
        }
    }

    /// Returns true if all fields are verified
    pub fn all_verified(&self) -> bool {
        matches!(self.first_commit_time, VerificationResult::Verified)
            && matches!(self.last_commit_time, VerificationResult::Verified)
    }

    /// Returns true if all fields are verified or skipped
    pub fn all_verified_or_skipped(&self) -> bool {
        (matches!(self.first_commit_time, VerificationResult::Verified)
            || matches!(self.first_commit_time, VerificationResult::Skipped))
            && (matches!(self.last_commit_time, VerificationResult::Verified)
                || matches!(self.last_commit_time, VerificationResult::Skipped))
    }
}

fn git_time_to_system_time(t: GitTime) -> SystemTime {
    let secs = t.seconds();
    if secs >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs((-secs) as u64)
    }
}

fn earliest_commit_time(repo: &Repository) -> Result<SystemTime, git2::Error> {
    let mut walk = repo.revwalk()?;
    // Walk commits by time to be efficient; still keep a min to be robust
    walk.set_sorting(Sort::TIME | Sort::REVERSE)?;
    walk.push_head()?;

    let mut earliest: Option<SystemTime> = None;
    for oid in walk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let t = git_time_to_system_time(commit.time());
        earliest = Some(match earliest {
            Some(curr) => {
                if t < curr {
                    t
                } else {
                    curr
                }
            }
            None => t,
        });
    }

    earliest.ok_or_else(|| git2::Error::from_str("repository has no commits"))
}

fn latest_commit_time(repo: &Repository) -> Result<SystemTime, git2::Error> {
    let mut walk = repo.revwalk()?;
    // Walk commits by time to be efficient; still keep a max to be robust
    walk.set_sorting(Sort::TIME)?;
    walk.push_head()?;

    let mut latest: Option<SystemTime> = None;
    for oid in walk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let t = git_time_to_system_time(commit.time());
        latest = Some(match latest {
            Some(curr) => {
                if t > curr {
                    t
                } else {
                    curr
                }
            }
            None => t,
        });
    }

    latest.ok_or_else(|| git2::Error::from_str("repository has no commits"))
}

fn verify_time(
    constraint: Option<&Range<SystemTime>>,
    actual_time: Result<SystemTime, git2::Error>,
) -> VerificationResult {
    match constraint {
        Some(range) => match actual_time {
            Ok(t) => {
                if range.contains(&t) {
                    VerificationResult::Verified
                } else {
                    VerificationResult::Failed(FailureReason::TimeNotInRange(t))
                }
            }
            Err(e) => VerificationResult::Failed(FailureReason::GitError(e)),
        },
        None => VerificationResult::Skipped,
    }
}

/// Check the repository at `path` against the given constraints.
pub fn check_metadata_at_path<P: AsRef<Path>>(
    path: P,
    constraints: MetadataConstraints,
) -> MetadataVerificationResult {
    match Repository::open(path) {
        Ok(repo) => check_metadata(&repo, constraints),
        Err(e) => MetadataVerificationResult::new(
            VerificationResult::Failed(FailureReason::GitError(e)),
            VerificationResult::Failed(FailureReason::GitError(git2::Error::from_str(
                "failed to open repository (see first error)",
            ))),
        ),
    }
}

/// Check the repository against the given constraints.
pub fn check_metadata(
    repo: &Repository,
    constraints: MetadataConstraints,
) -> MetadataVerificationResult {
    let first_result = verify_time(
        constraints.first_commit_time.as_ref(),
        earliest_commit_time(repo),
    );
    let last_result = verify_time(
        constraints.last_commit_time.as_ref(),
        latest_commit_time(repo),
    );
    MetadataVerificationResult::new(first_result, last_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_tools::verification::FailureReason::TimeNotInRange;
    use std::fs;
    use std::io::Write;

    fn unique_temp_dir() -> std::path::PathBuf {
        use rand::{RngCore, rng};
        let mut rng = rng();
        let mut bytes = [0u8; 8];
        rng.fill_bytes(&mut bytes);
        let unique = u64::from_le_bytes(bytes);
        let mut path = std::env::temp_dir();
        path.push(format!("git_meta_test_{}", unique));
        path
    }

    fn init_repo_with_one_commit() -> (std::path::PathBuf, Repository, SystemTime) {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).expect("create temp dir");
        let repo = Repository::init(&dir).expect("init repo");

        // create a file
        let file_path = dir.join("README.md");
        let mut f = fs::File::create(&file_path).expect("create file");
        writeln!(f, "hello").ok();
        f.sync_all().ok();

        // add and commit
        let mut index = repo.index().expect("index");
        index
            .add_path(std::path::Path::new("README.md"))
            .expect("add path");
        let tree_id = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        index.write().ok();

        // set signature
        let sig = git2::Signature::now("tester", "tester@example.com").expect("sig");
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .expect("commit");
        let commit = repo.find_commit(oid).expect("find commit");
        let t = git_time_to_system_time(commit.time());
        // Ensure any borrows tied to `repo` are dropped before returning `repo`
        drop(tree);
        drop(commit);
        (dir, repo, t)
    }

    #[test]
    fn test_create_empty_metadata_constraints() {
        let c = MetadataConstraints::new_empty();
        assert!(c.first_commit_time.is_none());
        assert!(c.last_commit_time.is_none());
    }

    #[test]
    fn test_create_metadata_constraints() {
        let now = SystemTime::now();
        let before = now - Duration::from_secs(3600);
        let after = now + Duration::from_secs(3600);
        let c = MetadataConstraints::new(Some(before..after), Some(before..after));
        assert_eq!(c.first_commit_time, Some(before..after));
        assert_eq!(c.last_commit_time, Some(before..after));
    }

    #[test]
    fn test_check_metadata_all_skipped() {
        let (_dir, repo, _t) = init_repo_with_one_commit();
        let c = MetadataConstraints::new_empty();
        let res = check_metadata(&repo, c);
        assert!(matches!(res.first_commit_time, VerificationResult::Skipped));
        assert!(matches!(res.last_commit_time, VerificationResult::Skipped));
        assert!(res.all_verified_or_skipped());
    }

    #[test]
    fn test_check_metadata_verified_ranges() {
        let (_dir, repo, t) = init_repo_with_one_commit();
        let dur = Duration::from_secs(5);
        let c = MetadataConstraints::new(Some((t - dur)..(t + dur)), Some((t - dur)..(t + dur)));
        let res = check_metadata(&repo, c);
        assert!(matches!(
            res.first_commit_time,
            VerificationResult::Verified
        ));
        assert!(matches!(res.last_commit_time, VerificationResult::Verified));
        assert!(res.all_verified());
    }

    #[test]
    fn test_check_metadata_out_of_range_fails() {
        let (_dir, repo, t) = init_repo_with_one_commit();
        let c = MetadataConstraints::new(
            Some((t + Duration::from_secs(10))..(t + Duration::from_secs(20))),
            Some((t + Duration::from_secs(10))..(t + Duration::from_secs(20))),
        );
        let res = check_metadata(&repo, c);
        match res.first_commit_time {
            VerificationResult::Failed(TimeNotInRange(actual)) => assert_eq!(actual, t),
            other => panic!("expected Failed(TimeNotInRange) for first, got {:?}", other),
        }
        match res.last_commit_time {
            VerificationResult::Failed(TimeNotInRange(actual)) => assert_eq!(actual, t),
            other => panic!("expected Failed(TimeNotInRange) for last, got {:?}", other),
        }
        assert!(!res.all_verified());
        assert!(!res.all_verified_or_skipped());
    }
}
