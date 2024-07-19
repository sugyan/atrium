use bsky_cli::{Command, Runner};
use clap::Parser;
use std::fmt::Debug;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "https://bsky.social")]
    pds_host: String,
    /// Limit the number of items returned
    #[arg(
        short,
        long,
        default_value = "10",
        value_parser = clap::value_parser!(u8).range(1..=100)
    )]
    limit: u8,
    /// Debug print
    #[arg(short, long)]
    debug: bool,
    #[command(subcommand)]
    // command: Command,
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    Ok(Runner::new(
        args.pds_host,
        args.limit.try_into()?,
        args.debug,
        matches!(args.command, Command::Login(_)),
    )
    .await?
    .run(args.command)
    .await?)
}
