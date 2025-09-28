mod file_tools;
mod git_tools;
mod zip_tools;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    zip: String,

    #[arg(short, long)]
    repo: String,

    #[arg(short, long, required = true, num_args = 0..)]
    usernames: Vec<String>,
}

fn main() {
    let args = Args::parse();

    println!("{}", args.zip);
    println!("{}", args.repo);
    for u in args.usernames {
        println!("{}", u);
    }
}
