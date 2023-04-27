use atrium_api::app::bsky as app_bsky;
use atrium_api::com::atproto as com_atproto;

use app_bsky::actor::get_profile::{GetProfile, Parameters as GetProfileParameters};
use app_bsky::feed::get_author_feed::{GetAuthorFeed, Parameters as GetAuthorFeedParameters};
use app_bsky::feed::get_post_thread::{GetPostThread, Parameters as GetPostThreadParameters};
use app_bsky::feed::get_timeline::{GetTimeline, Parameters as GetTimelineParameters};
use app_bsky::feed::post::Record as PostRecord;
use app_bsky::feed::repost::Record as RepostRecord;
use app_bsky::graph::get_followers::{GetFollowers, Parameters as GetFollowersParameters};
use app_bsky::graph::get_follows::{GetFollows, Parameters as GetFollowsParameters};
use atrium_api::records::Record;
use atrium_xrpc::XrpcReqwestClient;
use chrono::Utc;
use clap::{Parser, Subcommand};
use com_atproto::repo::create_record::{CreateRecord, Input as CreateRecordInput};
use com_atproto::repo::get_record::{GetRecord, Parameters as GetRecordParameters};
use com_atproto::server::create_session::{CreateSession, Input as CreateSessionInput};
use com_atproto::server::get_session::GetSession;
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
    /// Create a new record (post, repost)
    #[command(subcommand)]
    CreateRecord(CreateRecordCommand),
    /// Get current session info
    GetSession,
    /// Get a profile of an actor (default: current session)
    GetProfile { actor: Option<String> },
    /// Get record
    GetRecord { uri: String, cid: Option<String> },
    /// Get timeline
    GetTimeline,
    /// Get following of an actor (default: current session)
    GetFollows { actor: Option<String> },
    /// Get followers of an actor (default: current session)
    GetFollowers { actor: Option<String> },
    /// Get a feed of an author (default: current session)
    GetAuthorFeed { author: Option<String> },
    /// Get a post thread
    GetPostThread { uri: String },
}

#[derive(Subcommand, Debug)]
enum CreateRecordCommand {
    /// Create a post
    Post { text: String },
    /// Create a repost
    Repost { uri: String },
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
        Command::CreateRecord(record) => match record {
            CreateRecordCommand::Post { text } => print(
                client
                    .create_record(CreateRecordInput {
                        collection: String::from("app.bsky.feed.post"),
                        record: Record::AppBskyFeedPost(PostRecord {
                            created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            embed: None,
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
                    .await?,
                debug,
            )?,
            CreateRecordCommand::Repost { uri } => {
                let ru = RecordUri::try_from(uri.as_str())?;
                let record = client
                    .get_record(GetRecordParameters {
                        cid: None,
                        collection: ru.collection,
                        repo: ru.did,
                        rkey: ru.rkey,
                    })
                    .await?;
                print(
                    client
                        .create_record(CreateRecordInput {
                            collection: String::from("app.bsky.feed.repost"),
                            record: Record::AppBskyFeedRepost(RepostRecord {
                                created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                                subject: com_atproto::repo::strong_ref::Main {
                                    cid: record.cid.unwrap(),
                                    uri,
                                },
                            }),
                            repo: session.did,
                            rkey: None,
                            swap_commit: None,
                            validate: None,
                        })
                        .await?,
                    debug,
                )?
            }
        },
        Command::GetSession => print(client.get_session().await?, debug)?,
        Command::GetProfile { actor } => print(
            client
                .get_profile(GetProfileParameters {
                    actor: actor.unwrap_or(session.did),
                })
                .await?,
            debug,
        )?,
        Command::GetRecord { uri, cid } => {
            let ru = RecordUri::try_from(uri.as_str())?;
            print(
                client
                    .get_record(GetRecordParameters {
                        cid,
                        collection: ru.collection,
                        repo: ru.did,
                        rkey: ru.rkey,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetTimeline => print(
            client
                .get_timeline(GetTimelineParameters {
                    algorithm: None,
                    cursor: None,
                    limit: Some(25),
                })
                .await?,
            debug,
        )?,
        Command::GetFollows { actor } => print(
            client
                .get_follows(GetFollowsParameters {
                    actor: actor.unwrap_or(session.did),
                    cursor: None,
                    limit: Some(25),
                })
                .await?,
            debug,
        )?,
        Command::GetFollowers { actor } => print(
            client
                .get_followers(GetFollowersParameters {
                    actor: actor.unwrap_or(session.did),
                    cursor: None,
                    limit: Some(25),
                })
                .await?,
            debug,
        )?,
        Command::GetAuthorFeed { author } => print(
            client
                .get_author_feed(GetAuthorFeedParameters {
                    actor: author.unwrap_or(session.did),
                    cursor: None,
                    limit: Some(25),
                })
                .await?,
            debug,
        )?,
        Command::GetPostThread { uri } => print(
            client
                .get_post_thread(GetPostThreadParameters { depth: None, uri })
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

#[derive(Debug)]
struct RecordUri {
    did: String,
    collection: String,
    rkey: String,
}

impl TryFrom<&str> for RecordUri {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value
            .strip_prefix("at://did:plc:")
            .ok_or(r#"record uri must start with "at://did:plc:""#)?
            .splitn(3, '/')
            .collect::<Vec<_>>();
        Ok(Self {
            did: format!("did:plc:{}", parts[0]),
            collection: parts[1].to_string(),
            rkey: parts[2].to_string(),
        })
    }
}
