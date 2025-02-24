// mosvps
// Copyright (C) 2025 Yusei Honzawa

use clap::Parser;



#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    prefix: String,
    suffix: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let prefix = args.prefix.replace("\"","\\\"").replace("\\","\\\\");
    let suffix = args.suffix.replace("\"","\\\"").replace("\\","\\\\");

    println!("a\"{}\"a$post-text$a\"{}\"", prefix, suffix);

    Ok(())
}