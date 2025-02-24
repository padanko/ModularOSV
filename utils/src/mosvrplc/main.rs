// mosvps
// Copyright (C) 2025 Yusei Honzawa

use clap::Parser;



#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'f', long = "from")]
    from: String,
    #[arg(short = 't', long = "to")]
    to: String,
    #[arg(short = 'c', long = "count")]
    count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let from = args.from.replace("\"","\\\"").replace("\\","\\\\");
    let to = args.to.replace("\"","\\\"").replace("\\","\\\\");
    let count = args.count;

    let from_length: usize = from.chars().count();
    
    println!("^Lo^*{count};{{^IF^(^CT^\"{from}\"){{s\"{from}\"^Lo^*{from_length};{{fr}}a\"{to}\"}}{{q}}}}");

    Ok(())
}