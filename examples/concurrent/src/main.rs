use atrium_api::agent::AtpAgent;
use atrium_api::xrpc::client::reqwest::ReqwestClient;
use clap::Parser;
use futures::future::join_all;
use std::sync::Arc;

/// Simple program to concurrently fetch data by ATrium API agent.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Identifier of the login user.
    #[arg(short, long)]
    identifier: String,
    /// App password of the login user.
    #[arg(short, long)]
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let agent = Arc::new(AtpAgent::new(ReqwestClient::new(
        "https://bsky.social".into(),
    )));
    agent.login(&args.identifier, &args.password).await?;

    let actors = ["bsky.app", "atproto.com", "safety.bsky.app"];
    let handles = actors
        .iter()
        .map(|&actor| {
            let agent = Arc::clone(&agent);
            tokio::spawn(async move {
                agent
                    .api
                    .app
                    .bsky
                    .actor
                    .get_profile(atrium_api::app::bsky::actor::get_profile::Parameters {
                        actor: actor.into(),
                    })
                    .await
            })
        })
        .collect::<Vec<_>>();
    let results = join_all(handles).await;
    for (actor, result) in actors.iter().zip(results) {
        println!("{actor}: {:#?}", result?);
    }
    Ok(())
}
