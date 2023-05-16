use atrium_api::app::bsky::feed::post::Record;
use atrium_xrpc_server::stream::frames::{Frame, MessageEnum};
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut stream, _) =
        connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;

    while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
        match Frame::try_from(message.as_slice())? {
            Frame::Message(message) => match message.body {
                MessageEnum::Commit(commit) => {
                    // println!("{:?}: {:?}", commit.commit, commit.ops);
                    if let Some(op) = commit.ops.iter().find(|op| {
                        op.action == "create" && op.path.starts_with("app.bsky.feed.post")
                    }) {
                        let (items, _) =
                            rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
                        if let Some((cid, item)) = items.first() {
                            assert_eq!(Some(*cid), op.cid);
                            if let Ok(value) =
                                ciborium::de::from_reader::<Record, _>(&mut item.as_slice())
                            {
                                println!("{}: {}", value.created_at, value.text.replace('\n', " "));
                            } else {
                                // TODO
                            }
                        }
                    }
                }
            },
            Frame::Error(err) => panic!("{err:?}"),
        }
    }
    Ok(())
}
