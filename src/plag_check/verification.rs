//The plagiarism verification result
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum VerificationResult {
    //The f64 will contain a similarity percentage
    Verified(f64),

    //Copydetect was not able to automatically verify due to lack of supported language
    ManualRequired,
}

impl VerificationResult {
    pub fn get_percent(&self) -> f64 {
        match self {
            VerificationResult::Verified(x) => return *x,
            VerificationResult::ManualRequired => return -1.0,
        }
    }
}
