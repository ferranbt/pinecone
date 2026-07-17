use clap::{Parser, Subcommand};
use pine_core::{PineVersion, VersionError};

#[derive(Parser)]
#[command(name = "pine-reference")]
#[command(about = "Pine Script reference documentation tool", long_about = None)]
struct Cli {
    #[arg(long, global = true, default_value_t = PineVersion::LATEST.number())]
    pine_version: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download the reference page HTML from TradingView and convert to markdown
    Download,
    /// Query the reference (lists if prefix, shows content if exact match)
    Query {
        /// Optional path (e.g., "Variables", "Variables.bar", "Variables.ask")
        path: Option<String>,
    },
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    let version = PineVersion::from_number(cli.pine_version)
        .ok_or(VersionError::Unsupported(cli.pine_version))?;

    match &cli.command {
        Commands::Download => {
            pine_reference::download_and_save_reference(version)?;
        }
        Commands::Query { path } => match pine_reference::query(version, path.as_deref())? {
            pine_reference::QueryResult::List(items) => {
                for item in items {
                    println!("{}", item);
                }
            }
            pine_reference::QueryResult::Content(content) => {
                println!("{}", content);
            }
        },
    }

    Ok(())
}
