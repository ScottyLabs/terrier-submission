mod file_tools;
mod git_tools;
mod zip_tools;

use clap::Parser;
use std::vec::Vec;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use serde_json::{Value};
use chrono::{DateTime, Utc};

#[derive(Debug)]
struct Data {
    zip: String,
    repo: String,
    usernames: Vec<String>,
    start_time: DateTime<Utc>,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, required = true)]
    path: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(args.path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let serde_data: Value = serde_json::from_str(&contents)?;

    let serde_path = serde_data["zip"]
        .as_str()
        .unwrap()
        .to_string();

    let path = Path::new(&serde_path);
    if !path.exists() {
        return Err("Zip path provided does not exist.".into());
    }

    let serde_usernames = serde_data["usernames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let start_time_str = serde_data["start_time"]
        .as_str()
        .unwrap();

    let serde_date: DateTime<Utc> = start_time_str
        .parse()
        .map_err(|_| "Failed to parse start time string")?;

    let data = Data {
        zip: serde_path,
        repo: serde_data["repo"]
            .as_str()
            .unwrap()
            .to_string(),
        usernames: serde_usernames,
        start_time: serde_date,
    };

    println!("{:?}", &data);

    return Ok(());
}