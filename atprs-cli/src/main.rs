use atprs_xrpc::{Client, CreateRecordInput, CreateSessionInput, CreateSessionOutput, Record};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use toml::{Table, Value};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    GetProfile,
    CreateRecord { text: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let value = fs::read_to_string(args.config)?.parse::<Table>()?;
    if let (Some(Value::String(identifier)), Some(Value::String(password))) =
        (value.get("identifier"), value.get("password"))
    {
        run(identifier, password, args.command).await?;
    } else {
        panic!("invalid config");
    }
    Ok(())
}

async fn run(
    identifier: &str,
    password: &str,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client {};
    let session = client
        .create_session(CreateSessionInput {
            identifier: identifier.to_string(),
            password: password.to_string(),
        })
        .await?
        .error_for_status()?
        .json::<CreateSessionOutput>()
        .await?;
    let result = match command {
        Command::GetProfile => {
            client
                .get_profile(session.handle, session.access_jwt)
                .await?
                .json::<serde_json::Value>()
                .await?
        }
        Command::CreateRecord { text } => {
            client
                .create_record(
                    CreateRecordInput {
                        repo: session.did,
                        collection: String::from("app.bsky.feed.post"),
                        record: Record::FeedPost(text),
                    },
                    session.access_jwt,
                )
                .await?
                .json::<serde_json::Value>()
                .await?
        }
    };
    println!("{result:?}");
    Ok(())
}
