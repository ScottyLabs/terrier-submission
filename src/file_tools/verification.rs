use std::time::SystemTime;

/// The reason why a verification failed
#[derive(Debug)]
pub enum FailureReason {
    /// An I/O error occurred while trying to access the file system
    IoError(std::io::Error),
    /// The actual time is not within the specified range
    /// The contained `SystemTime` is the actual time that failed the check
    TimeNotInRange(SystemTime),
}

/// The result of verifying a single field against its constraint
#[derive(Debug)]
pub enum VerificationResult {
    /// This field satisfied the constraint
    Verified,
    /// This field was not verified because the constraint was not set
    Skipped,
    /// This field failed to satisfy the constraint
    Failed(FailureReason),
}