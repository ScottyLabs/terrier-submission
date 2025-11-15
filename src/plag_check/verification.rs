//The plagiarism verification result
pub enum VerificationResult {
    //The f64 will contain a similarity percentage
    Verified(f64),

    //The user has not installed all the necessary packages
    PrerequisitesNotMet,
}
