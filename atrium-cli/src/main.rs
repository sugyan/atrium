use atrium_api::app::bsky::actor::get_profile::{GetProfile, Parameters as GetProfileParameters};
use atrium_api::app::bsky::feed::get_timeline::{GetTimeline, Parameters as GetTimelineParameters};
use atrium_api::app::bsky::feed::post::Record as PostRecord;
use atrium_api::com::atproto::repo::create_record::{CreateRecord, Input as CreateRecordInput};
use atrium_api::com::atproto::server::create_session::{
    CreateSession, Input as CreateSessionInput,
};
use atrium_api::com::atproto::server::get_session::GetSession;
use atrium_api::records::Record;
use atrium_xrpc::XrpcReqwestClient;
use chrono::Utc;
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
    CreateRecord { text: String },
    GetProfile { actor: String },
    GetSession,
    GetTimeline,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let value = fs::read_to_string(args.config)?.parse::<Table>()?;
    if let (Some(Value::String(identifier)), Some(Value::String(password))) =
        (value.get("identifier"), value.get("password"))
    {
        run(
            args.pds_host,
            identifier.to_string(),
            password.to_string(),
            args.command,
        )
        .await?;
    } else {
        panic!("invalid config");
    }
    Ok(())
}

async fn run(
    host: String,
    identifier: String,
    password: String,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = XrpcReqwestClient::new(host);
    let session = client
        .create_session(CreateSessionInput {
            identifier,
            password,
        })
        .await?;
    client.set_auth(session.access_jwt);
    match command {
        Command::CreateRecord { text } => {
            println!(
                "{:#?}",
                client
                    .create_record(CreateRecordInput {
                        collection: String::from("app.bsky.feed.post"),
                        record: Record::AppBskyFeedPost(PostRecord {
                            created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            entities: None,
                            facets: None,
                            reply: None,
                            text,
                        }),
                        repo: session.did,
                        rkey: None,
                        swap_commit: None,
                        validate: None,
                    })
                    .await?
            );
        }
        Command::GetProfile { actor } => {
            println!(
                "{:#?}",
                client.get_profile(GetProfileParameters { actor }).await?
            );
        }
        Command::GetSession => {
            println!("{:#?}", client.get_session().await?);
        }
        Command::GetTimeline => {
            println!(
                "{:#?}",
                client
                    .get_timeline(GetTimelineParameters {
                        algorithm: None,
                        cursor: None,
                        limit: Some(25),
                    })
                    .await?
            );
        }
    };

    Ok(())
}
