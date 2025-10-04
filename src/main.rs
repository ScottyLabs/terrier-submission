mod file_tools;
mod git_tools;
mod zip_tools;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    zip: String,
    repo: String,
    usernames: Vec<String>,
    start_time: u64,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, required = true)]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file =
        File::open(&args.path).map_err(|_e| format!("JSON file {} does not exist.", args.path))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: ConfigData = serde_json::from_str(&contents)?;

    println!("{:?}", &data);

    Ok(())
}
