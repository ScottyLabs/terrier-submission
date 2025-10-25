use serde::Serialize;
use serde::ser::SerializeStruct;
use std::time::SystemTime;

/// The reason why a verification failed
#[derive(Debug)]
pub enum FailureReason {
    /// An error occurred while accessing the Git repository
    GitError(git2::Error),
    /// The actual time is not within the specified range
    /// The contained `SystemTime` is the actual time that failed the check
    TimeNotInRange(SystemTime),
    /// The usernames don't match the expected contributors
    /// The contained `Vec<String>` is the list of unexpected contributors
    UsernameMismatch(Vec<String>),
}

impl Serialize for FailureReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            FailureReason::GitError(e) => {
                // Object with two fields: type, message
                let mut state = serializer.serialize_struct("FailureReason", 2)?;
                state.serialize_field("errorType", "GitError")?;
                state.serialize_field("errorMessage", &e.message())?;
                state.end()
            }
            FailureReason::TimeNotInRange(t) => {
                let duration = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                let secs = duration.as_secs() as i64;
                let mut state = serializer.serialize_struct("FailureReason", 2)?;
                state.serialize_field("errorType", "TimeNotInRange")?;
                state.serialize_field("actualTime", &secs)?;
                state.end()
            }
            FailureReason::UsernameMismatch(unexpected) => {
                let mut state = serializer.serialize_struct("FailureReason", 2)?;
                state.serialize_field("errorType", "UsernameMismatch")?;
                state.serialize_field("unexpectedContributors", unexpected)?;
                state.end()
            }
        }
    }
}

/// The result of verifying a single field against its constraint
#[derive(Debug, Serialize)]
pub enum VerificationResult {
    /// This field satisfied the constraint
    Verified,
    /// This field was not verified because the constraint was not set
    Skipped,
    /// This field failed to satisfy the constraint
    Failed(FailureReason),
}
