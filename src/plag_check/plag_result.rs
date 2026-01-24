use crate::plag_check::verification::VerificationResult;

use regex::Regex;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct PlagiarismVerificationResult {
    pub result: VerificationResult,
    #[serde(skip_serializing)]
    pub report_path: Option<PathBuf>,
}

impl PlagiarismVerificationResult {
    pub fn new(similarity_percentage: Option<f64>, report_path: Option<PathBuf>) -> Self {
        let result = match similarity_percentage {
            Some(percentage) => VerificationResult::Verified(percentage),
            None => VerificationResult::ManualRequired,
        };
        Self {
            result,
            report_path,
        }
    }

    pub fn manual(report_path: Option<PathBuf>) -> Self {
        Self::new(None, report_path)
    }
}

pub fn copy_percentage_from_html(html_path: &Path) -> Option<f64> {
    let contents = std::fs::read_to_string(html_path).ok()?;
    let regex =
        Regex::new(r#"<b>Number above display threshold:</b>\s*\d+\s*\(([\d.]+)%\)<br><br>"#)
            .ok()?;
    let captures = regex.captures(&contents)?;
    let percent_str = captures.get(1)?.as_str();
    let percent = percent_str.parse::<f64>().ok()?;
    Some(percent / 100.0)
}
