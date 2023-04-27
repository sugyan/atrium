use atrium_api::app::bsky::actor::get_profile::{GetProfile, Parameters as GetProfileParameters};
use atrium_api::app::bsky::feed::get_timeline::{GetTimeline, Parameters as GetTimelineParameters};
use atrium_api::app::bsky::feed::post::Record as PostRecord;
use atrium_api::app::bsky::graph::get_followers::{
    GetFollowers, Parameters as GetFollowersParameters,
};
use atrium_api::app::bsky::graph::get_follows::{GetFollows, Parameters as GetFollowsParameters};
use atrium_api::com::atproto::repo::create_record::{CreateRecord, Input as CreateRecordInput};
use atrium_api::com::atproto::server::create_session::{
    CreateSession, Input as CreateSessionInput,
};
use atrium_api::com::atproto::server::get_session::GetSession;
use atrium_api::records::Record;
use atrium_xrpc::XrpcReqwestClient;
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::to_string_pretty;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use toml::{Table, Value};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "https://bsky.social")]
    pds_host: String,
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Debug print
    #[arg(short, long)]
    debug: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a new record
    CreateRecord { text: String },
    /// Get current session info
    GetSession,
    /// Get a profile of an actor (default: current session)
    GetProfile { actor: Option<String> },
    /// Get timeline
    GetTimeline,
    /// Get following of an actor (default: current session)
    GetFollows { actor: Option<String> },
    /// Get followers of an actor (default: current session)
    GetFollowers { actor: Option<String> },
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
            args.debug,
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
    debug: bool,
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
        Command::CreateRecord { text } => print(
            client
                .create_record(CreateRecordInput {
                    collection: String::from("app.bsky.feed.post"),
                    record: Record::AppBskyFeedPost(PostRecord {
                        text,
                        created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        ..Default::default()
                    }),
                    repo: session.did,
                    ..Default::default()
                })
                .await?,
            debug,
        )?,
        Command::GetSession => print(client.get_session().await?, debug)?,
        Command::GetProfile { actor } => print(
            client
                .get_profile(GetProfileParameters {
                    actor: actor.unwrap_or(session.did),
                })
                .await?,
            debug,
        )?,
        Command::GetTimeline => print(
            client
                .get_timeline(GetTimelineParameters {
                    limit: Some(25),
                    ..Default::default()
                })
                .await?,
            debug,
        )?,
        Command::GetFollows { actor } => print(
            client
                .get_follows(GetFollowsParameters {
                    actor: actor.unwrap_or(session.did),
                    ..Default::default()
                })
                .await?,
            debug,
        )?,
        Command::GetFollowers { actor } => print(
            client
                .get_followers(GetFollowersParameters {
                    actor: actor.unwrap_or(session.did),
                    ..Default::default()
                })
                .await?,
            debug,
        )?,
    }
    Ok(())
}

fn print(value: impl Debug + Serialize, debug: bool) -> Result<(), serde_json::Error> {
    if debug {
        println!("{:#?}", value);
    } else {
        println!("{}", to_string_pretty(&value)?);
    }
    Ok(())
}
