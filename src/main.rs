use clap::{Parser, ValueEnum};
use tokio::io::{stdin, AsyncBufReadExt, BufReader as AsyncBufReader};

mod display;
mod logs;
mod parse;
mod status;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Cargo,
    Errors,
    Verbose,
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.json {
        eprintln!("JSON mode is required. Use --json flag.");
        std::process::exit(1);
    }

    // Initialize display manager
    let mut display = display::DisplayManager::new(!args.no_color, args.level);

    // Read from stdin line by line
    let stdin = stdin();
    let reader = AsyncBufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if let Err(_) = parse::parse_nix_line(&line, &mut display).await {
            // Silently ignore parse errors
        }
    }

    Ok(())
}
