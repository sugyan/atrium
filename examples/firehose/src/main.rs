use futures::StreamExt;
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut stream, _) =
        connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;
    while let Some(message) = stream.next().await {
        println!("{message:?}");
    }
    Ok(())
}
