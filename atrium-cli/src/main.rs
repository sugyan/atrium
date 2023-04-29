use atrium_api::com::atproto::repo::strong_ref::Main as StrongRef;
use atrium_api::records::Record;
use atrium_xrpc::XrpcReqwestClient;
use chrono::Utc;
use clap::Parser;
use serde::Serialize;
use serde_json::to_string_pretty;
use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::Read;
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

#[derive(Parser, Debug)]
enum Command {
    /// Create a new record (post, repost, block)
    #[command(subcommand)]
    CreateRecord(CreateRecordCommand),
    /// Create a new app password
    CreateAppPassword { name: String },
    /// Delete record
    DeleteRecord { uri: String },
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
    /// Get a list of blocking actors
    GetBlocks,
    /// List app passwords
    ListAppPasswords,
    /// Revoke an app password
    RevokeAppPassword { name: String },
}

#[derive(Parser, Debug)]
enum CreateRecordCommand {
    /// Create a post
    Post(CreateRecordPostArgs),
    /// Create a repost
    Repost(CreateRecordRepostArgs),
    /// Block an actor
    Block(CreateRecordBlockArgs),
}

#[derive(Parser, Debug)]
struct CreateRecordPostArgs {
    /// Text of the post
    text: String,
    /// URI of the post to reply to
    #[arg(short, long)]
    reply: Option<String>,
    /// image files
    #[arg(short, long)]
    image: Option<Vec<PathBuf>>,
}

#[derive(Parser, Debug)]
struct CreateRecordRepostArgs {
    /// URI of the post to repost
    uri: String,
}

#[derive(Parser, Debug)]
struct CreateRecordBlockArgs {
    /// DID of an actor to block
    did: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let value = read_to_string(args.config)?.parse::<Table>()?;
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
    use atrium_api::com::atproto::server::create_session::{CreateSession, Input};
    let mut client = XrpcReqwestClient::new(host);
    let session = client
        .create_session(Input {
            identifier,
            password,
        })
        .await?;
    client.set_auth(session.access_jwt);
    match command {
        Command::CreateRecord(record) => {
            use atrium_api::com::atproto::repo::create_record::{CreateRecord, Input};
            use atrium_api::com::atproto::repo::get_record::{GetRecord, Parameters};
            let input = match record {
                CreateRecordCommand::Post(args) => {
                    use atrium_api::app::bsky::feed::post::{
                        Record as PostRecord, RecordEmbedEnum, ReplyRef,
                    };
                    let reply = if let Some(uri) = &args.reply {
                        let ru = RecordUri::try_from(uri.as_str())?;
                        let record = client
                            .get_record(Parameters {
                                cid: None,
                                collection: ru.collection,
                                repo: ru.did,
                                rkey: ru.rkey,
                            })
                            .await?;
                        let parent = StrongRef {
                            cid: record.cid.unwrap(),
                            uri: record.uri,
                        };
                        let mut root = parent.clone();
                        if let Record::AppBskyFeedPost(record) = record.value {
                            if let Some(reply) = record.reply {
                                root = reply.root;
                            }
                        };
                        Some(ReplyRef { parent, root })
                    } else {
                        None
                    };
                    let embed = if let Some(image) = &args.image {
                        use atrium_api::app::bsky::embed::images::{Image, Main as EmbedImages};
                        use atrium_api::com::atproto::repo::upload_blob::UploadBlob;
                        let mut images = Vec::with_capacity(image.len());
                        for path in image {
                            let mut input = Vec::new();
                            File::open(path)?.read_to_end(&mut input)?;
                            let output = client.upload_blob(input).await?;
                            images.push(Image {
                                alt: path
                                    .canonicalize()?
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .into(),
                                image: output.blob,
                            })
                        }
                        Some(Box::new(RecordEmbedEnum::AppBskyEmbedImagesMain(
                            EmbedImages { images },
                        )))
                    } else {
                        None
                    };
                    Input {
                        collection: String::from("app.bsky.feed.post"),
                        record: Record::AppBskyFeedPost(PostRecord {
                            created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            embed,
                            entities: None,
                            facets: None,
                            reply,
                            text: args.text,
                        }),
                        repo: session.did,
                        rkey: None,
                        swap_commit: None,
                        validate: None,
                    }
                }
                CreateRecordCommand::Repost(args) => {
                    use atrium_api::app::bsky::feed::repost::Record as RepostRecord;
                    let ru = RecordUri::try_from(args.uri.as_str())?;
                    let record = client
                        .get_record(Parameters {
                            cid: None,
                            collection: ru.collection,
                            repo: ru.did,
                            rkey: ru.rkey,
                        })
                        .await?;
                    Input {
                        collection: String::from("app.bsky.feed.repost"),
                        record: Record::AppBskyFeedRepost(RepostRecord {
                            created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            subject: StrongRef {
                                cid: record.cid.unwrap(),
                                uri: args.uri,
                            },
                        }),
                        repo: session.did,
                        rkey: None,
                        swap_commit: None,
                        validate: None,
                    }
                }
                CreateRecordCommand::Block(args) => {
                    use atrium_api::app::bsky::graph::block::Record as BlockRecord;
                    Input {
                        collection: String::from("app.bsky.graph.block"),
                        record: Record::AppBskyGraphBlock(BlockRecord {
                            created_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            subject: args.did,
                        }),
                        repo: session.did,
                        rkey: None,
                        swap_commit: None,
                        validate: None,
                    }
                }
            };
            print(client.create_record(input).await?, debug)?;
        }
        Command::CreateAppPassword { name } => {
            use atrium_api::com::atproto::server::create_app_password::{CreateAppPassword, Input};
            print(client.create_app_password(Input { name }).await?, debug)?
        }
        Command::DeleteRecord { uri } => {
            use atrium_api::com::atproto::repo::delete_record::{DeleteRecord, Input};
            let ru = RecordUri::try_from(uri.as_str())?;
            print(
                client
                    .delete_record(Input {
                        collection: ru.collection,
                        repo: ru.did,
                        rkey: ru.rkey,
                        swap_commit: None,
                        swap_record: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetSession => {
            use atrium_api::com::atproto::server::get_session::GetSession;
            print(client.get_session().await?, debug)?
        }
        Command::GetProfile { actor } => {
            use atrium_api::app::bsky::actor::get_profile::{GetProfile, Parameters};
            print(
                client
                    .get_profile(Parameters {
                        actor: actor.unwrap_or(session.did),
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetRecord { uri, cid } => {
            use atrium_api::com::atproto::repo::get_record::{GetRecord, Parameters};
            let ru = RecordUri::try_from(uri.as_str())?;
            print(
                client
                    .get_record(Parameters {
                        cid,
                        collection: ru.collection,
                        repo: ru.did,
                        rkey: ru.rkey,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetTimeline => {
            use atrium_api::app::bsky::feed::get_timeline::{GetTimeline, Parameters};
            print(
                client
                    .get_timeline(Parameters {
                        algorithm: None,
                        cursor: None,
                        limit: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetFollows { actor } => {
            use atrium_api::app::bsky::graph::get_follows::{GetFollows, Parameters};
            print(
                client
                    .get_follows(Parameters {
                        actor: actor.unwrap_or(session.did),
                        cursor: None,
                        limit: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetFollowers { actor } => {
            use atrium_api::app::bsky::graph::get_followers::{GetFollowers, Parameters};
            print(
                client
                    .get_followers(Parameters {
                        actor: actor.unwrap_or(session.did),
                        cursor: None,
                        limit: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetAuthorFeed { author } => {
            use atrium_api::app::bsky::feed::get_author_feed::{GetAuthorFeed, Parameters};
            print(
                client
                    .get_author_feed(Parameters {
                        actor: author.unwrap_or(session.did),
                        cursor: None,
                        limit: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::GetPostThread { uri } => {
            use atrium_api::app::bsky::feed::get_post_thread::{GetPostThread, Parameters};
            print(
                client
                    .get_post_thread(Parameters { depth: None, uri })
                    .await?,
                debug,
            )?
        }
        Command::GetBlocks => {
            use atrium_api::app::bsky::graph::get_blocks::{GetBlocks, Parameters};
            print(
                client
                    .get_blocks(Parameters {
                        cursor: None,
                        limit: None,
                    })
                    .await?,
                debug,
            )?
        }
        Command::ListAppPasswords => {
            use atrium_api::com::atproto::server::list_app_passwords::ListAppPasswords;
            print(client.list_app_passwords().await?, debug)?
        }
        Command::RevokeAppPassword { name } => {
            use atrium_api::com::atproto::server::revoke_app_password::{Input, RevokeAppPassword};
            print(client.revoke_app_password(Input { name }).await?, debug)?
        }
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
