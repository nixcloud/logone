use clap::Parser;
use logone::LogLevel;
use std::io::{stdin, BufRead, BufReader};

mod logone;
mod parser;
mod sinks;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable JSON parsing mode
    #[arg(short, long)]
    json: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Set log level for filtering messages
    #[arg(short, long, value_enum, default_value_t = LogLevel::Cargo)]
    level: LogLevel,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.json {
        eprintln!("JSON mode is required. Use --json flag.");
        std::process::exit(1);
    }

    // Initialize display manager
    let mut logone = logone::LogOne::new(!args.no_color, args.level);

    // Read from stdin line by line
    let stdin = stdin();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;
        if let Err(_) = parser::parse_nix_line(&line, &mut logone) {
            // Silently ignore parse errors
        }
    }

    Ok(())
}
