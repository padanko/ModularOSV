// mosvupdate
// Copyright (C) 2025 Yusei Honzawa

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about)]
struct Args {
    template: String,
    args: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    Ok(())
}