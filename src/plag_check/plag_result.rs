//Checks to see whether the files that a user has uploaded have similarities between past GitHub repositories
use crate::plag_check::verification::VerificationResult;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use regex::Regex;


pub struct PlagiarismVerificationResult {
    pub result: VerificationResult
}

impl PlagiarismVerificationResult {
    pub fn new(
        similarity_percentage: Option<f64>
    ) -> Self {
        if let Some(percentage) = similarity_percentage {
            Self {
                result: VerificationResult::Verified(percentage)
            }
        } else {
            Self {
                result: VerificationResult::PrerequisitesNotMet
            }
        }
    }
}

fn copy_percentage_from_html(html_path: Option<PathBuf>) -> Option<f64> {
    if let Some(path) = html_path {
        if !path.exists() {
            panic!("The HTML file path provided does not exist: {:?}", path);
        }

        let mut html_file = File::open(&path)
            .unwrap_or_else(|_| panic!("Failed to open HTML file: {:?}", path));

        let mut contents = String::new();
        html_file
            .read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Failed to read HTML file: {:?}", path));

        let re = Regex::new(
            r#"<b>Number above display threshold:</b>\s*\d+\s*\(([\d.]+)%\)<br><br>"#
        ).unwrap_or_else(|e| panic!("Invalid regex: {}", e));

        let caps = re
            .captures(&contents)
            .unwrap_or_else(|| panic!("Could not find the expected pattern in HTML file: {:?}", path));

        let y_str = caps.get(1)
            .unwrap_or_else(|| panic!("Could not extract captured group from HTML file: {:?}", path))
            .as_str();

        let y = y_str
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("Failed to parse captured number '{}' as f64", y_str));

        Some(y)
    } else {
        None
    }
}

