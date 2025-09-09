use clap::Parser;
use std::io::{self, BufRead, BufReader};
use tokio::io::{stdin, AsyncBufReadExt, BufReader as AsyncBufReader};

mod display;
mod logs;
mod misc;
mod parse;
mod stats;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable JSON parsing mode
    #[arg(short, long)]
    json: bool,

    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Show timing information
    #[arg(short, long)]
    timing: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Minimum time to show in seconds
    #[arg(long, default_value = "1")]
    min_time: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.json {
        eprintln!("JSON mode is required. Use --json flag.");
        std::process::exit(1);
    }

    // Initialize display manager
    let mut display =
        display::DisplayManager::new(args.verbose, !args.no_color, args.timing, args.debug);

    // Read from stdin line by line
    let stdin = stdin();
    let reader = AsyncBufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if let Err(e) = parse::parse_line(&line, &mut display).await {
            if args.debug {
                eprintln!("Parse error: {}", e);
            }
        }
    }

    Ok(())
}
