use atrium_api::app::bsky::feed::post::Record;
use atrium_xrpc_server::stream::frames::{Frame, MessageEnum};
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut stream, _) =
        connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;

    while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
        process_message(&message).await?;
    }
    Ok(())
}

async fn process_message(message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    match Frame::try_from(message)? {
        Frame::Message(message) => {
            match message.body {
                MessageEnum::Commit(commit) => {
                    for op in commit.ops {
                        let collection = op.path.split('/').next().expect("op.path is empty");
                        if op.action != "create" || collection != "app.bsky.feed.post" {
                            continue;
                        }
                        let (items, _) =
                            rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
                        if let Some((cid, item)) = items.first() {
                            assert_eq!(Some(*cid), op.cid);
                            if let Ok(value) =
                                ciborium::de::from_reader::<Record, _>(&mut item.as_slice())
                            {
                                println!("{}: {}", value.created_at, value.text);
                            } else {
                                // TODO
                            }
                        }
                    }
                }
            }
        }
        Frame::Error(err) => panic!("{err:?}"),
    }
    Ok(())
}
