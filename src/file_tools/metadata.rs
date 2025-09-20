use crate::file_tools::verification::VerificationResult;
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
struct MetadataVerificationResult {
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

pub fn check_metadata(metadata: Metadata, constraints: MetadataConstraints) -> MetadataVerificationResult {
    let modified_result = match constraints.modified {
        Some(range) => {
            if range.contains(&metadata.modified) {
                VerificationResult::Verified
            } else {
                VerificationResult::Failed(FailureReason::TimeNotInRange(metadata.modified))
            }
        }
    };

    let accessed_result = match constraints.accessed {
        Some(range) => {
            if range.contains(&metadata.accessed) {
                VerificationResult::Verified
            } else {
                VerificationResult::Failed(FailureReason::TimeNotInRange(metadata.accessed))
            }
        }
        None => VerificationResult::Skipped,
    };

    let created_result = match constraints.created {
        Some(range) => {
            if range.contains(&metadata.created) {
                VerificationResult::Verified
            } else {
                VerificationResult::Failed(FailureReason::TimeNotInRange(metadata.created))
            }
        }
        None => VerificationResult::Skipped,
    };

    MetadataVerificationResult::new(modified_result, accessed_result, created_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_tools::verification::FailureReason;
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

        let now = SystemTime::now();
        let result = MetadataVerificationResult::new(
            VerificationResult::Verified,
            VerificationResult::Skipped,
            VerificationResult::Verified,
        );
        assert!(!result.all_verified());
    }

    #[test]
    fn test_metadata_verification_result_some_skipped() {
        let now = SystemTime::now();
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
}
