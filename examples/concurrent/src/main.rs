use atrium_api::agent::{
    atp_agent::{store::MemorySessionStore, CredentialSession},
    Agent,
};
use atrium_xrpc_client::reqwest::ReqwestClient;
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
    let session = CredentialSession::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );
    session.login(&args.identifier, &args.password).await?;
    let agent = Arc::new(Agent::new(session));
    let actors = ["bsky.app", "atproto.com", "safety.bsky.app"];
    let handles = actors
        .iter()
        .map(|&actor| {
            println!("fetching profile of {actor}...");
            let agent = Arc::clone(&agent);
            tokio::spawn(async move {
                agent
                    .api
                    .app
                    .bsky
                    .actor
                    .get_profile(
                        atrium_api::app::bsky::actor::get_profile::ParametersData {
                            actor: actor.parse().expect("invalid actor"),
                        }
                        .into(),
                    )
                    .await
            })
        })
        .collect::<Vec<_>>();
    let results = join_all(handles).await;
    println!("{} profiles fetched!", results.len());
    for (actor, result) in actors.iter().zip(results) {
        let result = result??;
        println!("{actor} ({}) {:?}", result.did.as_ref(), result.description);
    }
    Ok(())
}
