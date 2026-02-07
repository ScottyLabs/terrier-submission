use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum VerificationResult {
    Verified(f64),

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
