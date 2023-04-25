use atprs_api::app::bsky::actor::get_profile::{GetProfile, Parameters};
use atprs_api::com::atproto::server::create_session::{CreateSession, Input};
use atprs_xrpc::XrpcReqwestClient;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use toml::{Table, Value};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "https://bsky.social")]
    pds_host: String,
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    GetProfile { actor: String },
    CreateRecord { text: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let value = fs::read_to_string(args.config)?.parse::<Table>()?;
    if let (Some(Value::String(identifier)), Some(Value::String(password))) =
        (value.get("identifier"), value.get("password"))
    {
        run(args.pds_host, identifier, password, args.command).await?;
    } else {
        panic!("invalid config");
    }
    Ok(())
}

async fn run(
    host: String,
    identifier: &str,
    password: &str,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = XrpcReqwestClient::new(host);
    let session = client
        .create_session(Input {
            identifier: identifier.to_string(),
            password: password.to_string(),
        })
        .await?;
    client.set_auth(session.access_jwt);
    match command {
        Command::GetProfile { actor } => {
            println!("{:?}", client.get_profile(Parameters { actor }).await?);
        }
        Command::CreateRecord { text: _ } => {
            todo!()
        }
    };

    Ok(())
}
