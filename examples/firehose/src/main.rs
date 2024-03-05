use anyhow::{anyhow, Result};
use atrium_api::app::bsky::feed::post::Record;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, NSID};
use atrium_api::types::{CidLink, Collection};
use chrono::Local;
use firehose::stream::frames::Frame;
use firehose::subscription::{CommitHandler, Subscription};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

struct RepoSubscription {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl RepoSubscription {
    async fn new(bgs: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, _) = connect_async(format!("wss://{bgs}/xrpc/{NSID}")).await?;
        Ok(RepoSubscription { stream })
    }
    async fn run(&mut self, handler: impl CommitHandler) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(result) = self.next().await {
            if let Ok(Frame::Message(Some(t), message)) = result {
                if t.as_str() == "#commit" {
                    let commit = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
                    if let Err(err) = handler.handle_commit(&commit).await {
                        eprintln!("FAILED: {err:?}");
                    }
                }
            }
        }
        Ok(())
    }
}

impl Subscription for RepoSubscription {
    async fn next(&mut self) -> Option<Result<Frame, <Frame as TryFrom<&[u8]>>::Error>> {
        if let Some(Ok(Message::Binary(data))) = self.stream.next().await {
            Some(Frame::try_from(data.as_slice()))
        } else {
            None
        }
    }
}

struct Firehose;

impl CommitHandler for Firehose {
    async fn handle_commit(&self, commit: &Commit) -> Result<()> {
        for op in &commit.ops {
            let collection = op.path.split('/').next().expect("op.path is empty");
            if op.action != "create" || collection != atrium_api::app::bsky::feed::Post::NSID {
                continue;
            }
            let (items, _) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
            if let Some((_, item)) = items.iter().find(|(cid, _)| Some(CidLink(*cid)) == op.cid) {
                let record = serde_ipld_dagcbor::from_reader::<Record, _>(&mut item.as_slice())?;
                println!(
                    "{} - {}",
                    record.created_at.as_ref().with_timezone(&Local),
                    commit.repo.as_str()
                );
                for line in record.text.split('\n') {
                    println!("  {line}");
                }
            } else {
                return Err(anyhow!(
                    "FAILED: could not find item with operation cid {:?} out of {} items",
                    op.cid,
                    items.len()
                ));
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RepoSubscription::new("bsky.network")
        .await?
        .run(Firehose)
        .await
}
