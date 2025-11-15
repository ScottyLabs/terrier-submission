//The plagiarism verification result
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum VerificationResult {
    //The f64 will contain a similarity percentage
    Verified(f64),

    //The user has not installed all the necessary packages
    PrerequisitesNotMet,
}
