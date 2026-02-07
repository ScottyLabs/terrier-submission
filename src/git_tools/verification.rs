use serde::Serialize;
use serde::ser::SerializeStruct;
use std::time::SystemTime;

#[derive(Debug)]
pub enum FailureReason {
    GitError(git2::Error),
    TimeNotInRange(SystemTime),
    AdditionalUnauthorizedUsers(Vec<String>),
}

impl Serialize for FailureReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            FailureReason::GitError(e) => {
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
            FailureReason::AdditionalUnauthorizedUsers(unexpected) => {
                let mut state = serializer.serialize_struct("FailureReason", 2)?;
                state.serialize_field("errorType", "UsernameMismatch")?;
                state.serialize_field("unexpectedContributors", unexpected)?;
                state.end()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum VerificationResult {
    Verified,
    Skipped,
    Failed(FailureReason),
}
