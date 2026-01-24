use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum VerificationResult {
    Verified(f64),

    ManualRequired,
}
