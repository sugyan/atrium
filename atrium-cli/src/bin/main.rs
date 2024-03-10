use atrium_cli::runner::Runner;
use clap::Parser;
use std::fmt::Debug;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "https://bsky.social")]
    pds_host: String,
    /// Debug print
    #[arg(short, long)]
    debug: bool,
    #[command(subcommand)]
    // command: Command,
    command: atrium_cli::commands::Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    Ok(Runner::new(args.pds_host, args.debug)
        .await?
        .run(args.command)
        .await?)
}
