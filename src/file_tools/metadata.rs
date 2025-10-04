use crate::file_tools::verification::FailureReason;
use crate::file_tools::verification::VerificationResult;
use std::fs::Metadata;
use std::ops::Range;
use std::time::SystemTime;

/// A set of constraints for verifying whether a metadata satisfies certain conditions.
/// In each constraint, `None` means no constraint is applied.
#[derive(Debug, Clone)]
pub struct MetadataConstraints {
    /// Range of allowed modified time
    modified: Option<Range<SystemTime>>,
    /// Range of last accessed time
    accessed: Option<Range<SystemTime>>,
    /// Range of created time
    created: Option<Range<SystemTime>>,
}

impl MetadataConstraints {
    /// Create a new `MetadataConstraints` with no constraints set.
    pub fn new_empty() -> Self {
        Self {
            modified: None,
            accessed: None,
            created: None,
        }
    }

    /// Create a new `MetadataConstraints`
    pub fn new(
        modified: Option<Range<SystemTime>>,
        accessed: Option<Range<SystemTime>>,
        created: Option<Range<SystemTime>>,
    ) -> Self {
        Self {
            modified,
            accessed,
            created,
        }
    }
}

/// The result of verifying a file's metadata against a set of constraints.
#[derive(Debug)]
pub struct MetadataVerificationResult {
    /// Result of verifying the modified time
    pub modified: VerificationResult,
    /// Result of verifying the last accessed time
    pub accessed: VerificationResult,
    /// Result of verifying the created time
    pub created: VerificationResult,
}

impl MetadataVerificationResult {
    /// Create a new `MetadataVerificationResult`
    pub fn new(
        modified: VerificationResult,
        accessed: VerificationResult,
        created: VerificationResult,
    ) -> Self {
        Self {
            modified,
            accessed,
            created,
        }
    }

    /// Returns true if all fields are verified
    pub fn all_verified(&self) -> bool {
        matches!(self.modified, VerificationResult::Verified)
            && matches!(self.accessed, VerificationResult::Verified)
            && matches!(self.created, VerificationResult::Verified)
    }

    /// Returns true if all fields are verified or skipped
    pub fn all_verified_or_skipped(&self) -> bool {
        (matches!(self.modified, VerificationResult::Verified)
            || matches!(self.modified, VerificationResult::Skipped))
            && (matches!(self.accessed, VerificationResult::Verified)
                || matches!(self.accessed, VerificationResult::Skipped))
            && (matches!(self.created, VerificationResult::Verified)
                || matches!(self.created, VerificationResult::Skipped))
    }
}

fn verify_time<F>(constraint: Option<&Range<SystemTime>>, getter: F) -> VerificationResult
where
    F: FnOnce() -> Result<SystemTime, std::io::Error>,
{
    match constraint {
        Some(range) => match getter() {
            Ok(time) => {
                if range.contains(&time) {
                    VerificationResult::Verified
                } else {
                    VerificationResult::Failed(FailureReason::TimeNotInRange(time))
                }
            }
            Err(e) => VerificationResult::Failed(FailureReason::IoError(e)),
        },
        None => VerificationResult::Skipped,
    }
}

pub fn check_metadata(
    metadata: Metadata,
    constraints: MetadataConstraints,
) -> MetadataVerificationResult {
    let modified_result = verify_time(constraints.modified.as_ref(), || metadata.modified());
    let accessed_result = verify_time(constraints.accessed.as_ref(), || metadata.accessed());
    let created_result = verify_time(constraints.created.as_ref(), || metadata.created());

    MetadataVerificationResult::new(modified_result, accessed_result, created_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_tools::verification::FailureReason::TimeNotInRange;

    #[test]
    fn test_create_empty_metadata_constraints() {
        let constraints = MetadataConstraints::new_empty();
        assert!(constraints.modified.is_none());
        assert!(constraints.accessed.is_none());
        assert!(constraints.created.is_none());
    }

    #[test]
    fn test_create_metadata_constraints() {
        let now = SystemTime::now();
        let before = now - std::time::Duration::from_secs(3600);
        let after = now + std::time::Duration::from_secs(3600);
        let constraints = MetadataConstraints::new(
            Some(before..after),
            Some(before..after),
            Some(before..after),
        );
        assert!(constraints.modified.is_some());
        assert_eq!(constraints.modified, Some(before..after));
        assert!(constraints.accessed.is_some());
        assert_eq!(constraints.accessed, Some(before..after));
        assert!(constraints.created.is_some());
        assert_eq!(constraints.created, Some(before..after));
    }

    #[test]
    fn test_metadata_verification_result_all_verified() {
        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Verified,
            VerificationResult::Verified,
        );
        assert!(result.all_verified());

        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Skipped,
            VerificationResult::Verified,
        );
        assert!(!result.all_verified());
    }

    #[test]
    fn test_metadata_verification_result_some_skipped() {
        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Verified,
            VerificationResult::Verified,
        );
        assert!(result.all_verified_or_skipped());

        let now = SystemTime::now();
        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Skipped,
            VerificationResult::Verified,
        );
        assert!(result.all_verified_or_skipped());

        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Skipped,
            VerificationResult::Failed(TimeNotInRange(now)),
        );
        assert!(!result.all_verified());
        assert!(!result.all_verified_or_skipped());
    }

    fn unique_temp_path() -> std::path::PathBuf {
        use rand::{RngCore, rng};
        let mut rng = rng();
        let mut bytes = [0u8; 8];
        rng.fill_bytes(&mut bytes);
        let unique = u64::from_le_bytes(bytes);
        let mut path = std::env::temp_dir();
        path.push(format!("check_metadata_test_{}.tmp", unique));
        path
    }

    fn create_temp_file_and_metadata() -> (std::path::PathBuf, Metadata) {
        use std::fs::{File, metadata as stat};
        use std::io::Write;
        let path = unique_temp_path();
        let mut f = File::create(&path).expect("create temp file");
        // Write a small payload to ensure the file exists and has times set
        f.write_all(b"meta-test").expect("write temp file");
        f.sync_all().ok();
        let md = stat(&path).expect("stat temp file");
        (path, md)
    }

    #[test]
    fn test_check_metadata_all_skipped() {
        let (path, md) = create_temp_file_and_metadata();
        let constraints = MetadataConstraints::new_empty();
        let res = check_metadata(md, constraints);
        assert!(matches!(res.modified, VerificationResult::Skipped));
        assert!(matches!(res.accessed, VerificationResult::Skipped));
        assert!(matches!(res.created, VerificationResult::Skipped));
        assert!(res.all_verified_or_skipped());
        // cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_check_metadata_all_verified_or_skipped() {
        let (path, md) = create_temp_file_and_metadata();
        let dur = std::time::Duration::from_secs(5);
        let m = md.modified().expect("metadata.modified");
        let a = md.accessed().expect("metadata.accessed");
        // Some platforms may not support created(); if so, don't constrain it.
        let created_ok = md.created().ok();

        let modified_range = (m - dur)..(m + dur);
        let accessed_range = (a - dur)..(a + dur);
        let created_range = created_ok.map(|c| (c - dur)..(c + dur));

        let constraints =
            MetadataConstraints::new(Some(modified_range), Some(accessed_range), created_range);
        let res = check_metadata(md, constraints);
        assert!(matches!(res.modified, VerificationResult::Verified));
        assert!(matches!(res.accessed, VerificationResult::Verified));
        match created_ok {
            Some(_) => assert!(matches!(res.created, VerificationResult::Verified)),
            None => assert!(matches!(res.created, VerificationResult::Skipped)),
        }
        assert!(res.all_verified_or_skipped());
        // cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_check_metadata_modified_out_of_range_fails() {
        let (path, md) = create_temp_file_and_metadata();
        let m = md.modified().expect("metadata.modified");
        // Range completely after the actual modified time
        let constraints = MetadataConstraints::new(
            Some(
                (m + std::time::Duration::from_secs(10))..(m + std::time::Duration::from_secs(20)),
            ),
            None,
            None,
        );
        let res = check_metadata(md, constraints);
        match res.modified {
            VerificationResult::Failed(FailureReason::TimeNotInRange(t)) => assert_eq!(t, m),
            other => panic!("expected Failed(TimeNotInRange), got {:?}", other),
        }
        assert!(matches!(res.accessed, VerificationResult::Skipped));
        assert!(matches!(res.created, VerificationResult::Skipped));
        // cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_check_metadata_accessed_out_of_range_fails() {
        let (path, md) = create_temp_file_and_metadata();
        let a = md.accessed().expect("metadata.accessed");
        let constraints = MetadataConstraints::new(
            None,
            Some(
                (a + std::time::Duration::from_secs(10))..(a + std::time::Duration::from_secs(20)),
            ),
            None,
        );
        let res = check_metadata(md, constraints);
        match res.accessed {
            VerificationResult::Failed(FailureReason::TimeNotInRange(t)) => assert_eq!(t, a),
            other => panic!(
                "expected Failed(TimeNotInRange) for accessed, got {:?}",
                other
            ),
        }
        assert!(matches!(res.modified, VerificationResult::Skipped));
        assert!(matches!(res.created, VerificationResult::Skipped));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_check_metadata_created_out_of_range_or_skipped() {
        let (path, md) = create_temp_file_and_metadata();
        let created = md.created().ok();
        let constraints = match created {
            Some(c) => MetadataConstraints::new(
                None,
                None,
                Some(
                    (c + std::time::Duration::from_secs(10))
                        ..(c + std::time::Duration::from_secs(20)),
                ),
            ),
            None => MetadataConstraints::new(None, None, None),
        };
        let res = check_metadata(md, constraints);
        match created {
            Some(c) => match res.created {
                VerificationResult::Failed(FailureReason::TimeNotInRange(t)) => assert_eq!(t, c),
                other => panic!(
                    "expected Failed(TimeNotInRange) for created, got {:?}",
                    other
                ),
            },
            None => assert!(matches!(res.created, VerificationResult::Skipped)),
        }
        assert!(matches!(res.modified, VerificationResult::Skipped));
        assert!(matches!(res.accessed, VerificationResult::Skipped));
        let _ = std::fs::remove_file(path);
    }
}
