use atrium_api::{client::AtpServiceClient, com::atproto::server::create_session::Input as CreateSessionRequest, com::atproto::server::create_session::Output as CreateSessionResponse};
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

const BLUESKY_DID: &str = "did:plc:z72i7hdynmk6r22z27h6tvur";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bluesky_username = dotenvy::var("BLUESKY_USERNAME").expect("BLUESKY_USERNAME must be set in the environment");
    let bluesky_password = dotenvy::var("BLUESKY_PASSWORD").expect("BLUESKY_PASSWORD must be set in the environment");

    let anon_agent = AtpServiceClient::new(ReqwestClient::new("https://bsky.social"));

    let create_session_response: CreateSessionResponse = anon_agent.service.com.atproto.server.create_session(CreateSessionRequest {
        identifier: bluesky_username.to_owned(),
        password: bluesky_password.to_owned(),
    })
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", create_session_response.access_jwt)).unwrap());

    let authed_agent = AtpServiceClient::new(ReqwestClientBuilder::new("https://bsky.social")
        .client(
            reqwest::ClientBuilder::new()
            .default_headers(headers)
                .use_rustls_tls()
                .build()?,
        )
        .build());
    
    let profile = authed_agent
        .service
        .app
        .bsky
        .actor
        .get_profile(atrium_api::app::bsky::actor::get_profile::Parameters {
            actor: BLUESKY_DID.to_owned()
        })
        .await?;

    println!("display name: {}", profile.display_name.unwrap());
    println!("handle: {}", profile.handle);
    println!("posts: {}", profile.posts_count.unwrap());
    println!("description: {}", profile.description.unwrap());

    Ok(())
}
