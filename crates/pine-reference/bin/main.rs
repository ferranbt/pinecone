use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pine-reference")]
#[command(about = "Pine Script reference documentation tool", long_about = None)]
struct Cli {
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

    match &cli.command {
        Commands::Download => {
            pine_reference::download_and_save_reference()?;
        }
        Commands::Query { path } => match pine_reference::query(path.as_deref())? {
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
