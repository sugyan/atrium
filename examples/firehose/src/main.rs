use atrium_api::app::bsky::feed::post::Record;
use atrium_api::com::atproto::sync::subscribe_repos::{Message, NSID};
use atrium_api::types::CidLink;
use firehose::stream::frames::Frame;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bgs = "bsky.network";
    let (mut stream, _) = connect_async(format!("wss://{bgs}/xrpc/{NSID}")).await?;
    while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
        process_message(&message).await.unwrap();
    }
    Ok(())
}

async fn process_message(message: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match Frame::try_from(message)? {
        Frame::Message(message) => {
            if let Message::Commit(commit) = message.body {
                for op in commit.ops {
                    let collection = op.path.split('/').next().expect("op.path is empty");
                    if op.action != "create" || collection != "app.bsky.feed.post" {
                        continue;
                    }
                    let (items, _) =
                        rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
                    if let Some((_, item)) =
                        items.iter().find(|(cid, _)| Some(CidLink(*cid)) == op.cid)
                    {
                        let record =
                            serde_ipld_dagcbor::from_reader::<Record, _>(&mut item.as_slice())?;
                        println!("{}: {}", record.created_at.as_ref(), record.text);
                    } else {
                        panic!(
                            "FAILED: could not find item with operation cid {:?} out of {} items",
                            op.cid,
                            items.len()
                        );
                    }
                }
            }
        }
        Frame::Error(err) => panic!("{err:?}"),
    }
    Ok(())
}
