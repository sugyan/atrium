use anyhow::{anyhow, Result};
use atrium_api::types::string::RecordKey;
use chrono::Local;
use firehose::stream::frames::Frame;
use firehose::subscription::{CommitHandler, Subscription};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use atrium_api::app::bsky::feed;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, NSID};
use atrium_api::types::Collection;
use atrium_repo::{blockstore::CarStore, Repository};

struct RepoSubscription {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl RepoSubscription {
    async fn new(bgs: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, _) = connect_async(format!("wss://{bgs}/xrpc/{NSID}")).await?;
        Ok(RepoSubscription { stream })
    }
    async fn run(&mut self, handler: impl CommitHandler) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let message = match self.next().await {
                Some(msg) => msg,
                None => continue, // Could be a websocket control message, etc.
            };

            match message {
                Ok(Frame::Message(Some(t), message)) => {
                    if t.as_str() == "#commit" {
                        let commit = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;

                        // Handle the commit on the websocket reader thread.
                        //
                        // N.B: You should ensure that your commit handler either executes as quickly as
                        // possible or offload processing to a separate thread. If you run too far behind,
                        // the firehose server _will_ terminate the connection!
                        if let Err(err) = handler.handle_commit(&commit).await {
                            eprintln!("FAILED: {err:?}");
                        }
                    }
                }
                Ok(Frame::Message(None, _msg)) => (),
                Ok(Frame::Error(_e)) => {
                    println!("received error frame");
                    break;
                }
                Err(e) => {
                    println!("error {e}");
                }
            }
        }
        Ok(())
    }
}

impl Subscription for RepoSubscription {
    async fn next(&mut self) -> Option<anyhow::Result<Frame>> {
        match self.stream.next().await {
            Some(Ok(Message::Binary(data))) => Some(Frame::try_from(data.as_slice())),
            Some(Ok(_)) | None => None,
            Some(Err(e)) => Some(Err(anyhow::Error::new(e))),
        }
    }
}

struct Firehose;

impl CommitHandler for Firehose {
    async fn handle_commit(&self, commit: &Commit) -> Result<()> {
        let mut repo = Repository::open(
            CarStore::open(std::io::Cursor::new(commit.blocks.as_slice())).await?,
            // N.B: This same CID is also specified inside of the `CarStore`, accessible
            // via `car.roots().next().unwrap()`.
            commit.commit.0,
        )
        .await?;

        for op in &commit.ops {
            let mut s = op.path.split('/');
            let collection = s.next().expect("op.path is empty");
            let rkey = s.next().expect("no record key");
            if op.action != "create" {
                continue;
            }

            let rkey = RecordKey::new(rkey.to_string()).expect("invalid record key");

            match collection {
                feed::Post::NSID => {
                    // N.B: We do _NOT_ read out the record using `op.cid` because that is insecure.
                    // It bypasses the MST, which means that we cannot ensure that the contents are
                    // signed by the owner of the repository.
                    // You will always want to read out records using the MST to ensure they haven't been
                    // tampered with.
                    if let Some(record) = repo.get::<feed::Post>(rkey).await? {
                        println!(
                            "{} - {} - {}",
                            record.created_at.as_ref().with_timezone(&Local),
                            commit.repo.as_str(),
                            op.path
                        );
                        for line in record.text.split('\n') {
                            println!("  {line}");
                        }
                    } else {
                        return Err(anyhow!(
                            "FAILED: could not find item with operation {}",
                            op.path
                        ));
                    }
                }
                _ => {
                    println!(
                        "{} - {} - {}",
                        commit.time.as_ref().with_timezone(&Local),
                        commit.repo.as_str(),
                        op.path
                    );
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RepoSubscription::new("bsky.network").await?.run(Firehose).await
}
