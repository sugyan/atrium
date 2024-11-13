use atrium_api::agent::Agent;
use atrium_identity::did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL};
use atrium_identity::handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig, DnsTxtResolver};
use atrium_oauth_client::store::state::MemoryStateStore;
use atrium_oauth_client::{
    AtprotoLocalhostClientMetadata, AuthorizeOptions, DefaultHttpClient, OAuthClient,
    OAuthClientConfig, OAuthResolverConfig,
};
use atrium_xrpc::http::Uri;
use hickory_resolver::TokioAsyncResolver;
use std::io::{stdin, stdout, BufRead, Write};
use std::sync::Arc;

struct HickoryDnsTxtResolver {
    resolver: TokioAsyncResolver,
}

impl Default for HickoryDnsTxtResolver {
    fn default() -> Self {
        Self {
            resolver: TokioAsyncResolver::tokio_from_system_conf()
                .expect("failed to create resolver"),
        }
    }
}

impl DnsTxtResolver for HickoryDnsTxtResolver {
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(self.resolver.txt_lookup(query).await?.iter().map(|txt| txt.to_string()).collect())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Arc::new(DefaultHttpClient::default());
    let config = OAuthClientConfig {
        client_metadata: AtprotoLocalhostClientMetadata {
            redirect_uris: vec!["http://127.0.0.1".to_string()],
        },
        keys: None,
        resolver: OAuthResolverConfig {
            did_resolver: CommonDidResolver::new(CommonDidResolverConfig {
                plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
                http_client: http_client.clone(),
            }),
            handle_resolver: AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
                dns_txt_resolver: HickoryDnsTxtResolver::default(),
                http_client: http_client.clone(),
            }),
            authorization_server_metadata: Default::default(),
            protected_resource_metadata: Default::default(),
        },
        state_store: MemoryStateStore::default(),
    };
    let client = OAuthClient::new(config)?;
    println!(
        "Authorization url: {}",
        client
            .authorize(
                std::env::var("HANDLE").unwrap_or(String::from("https://bsky.social")),
                AuthorizeOptions {
                    scopes: Some(vec![String::from("atproto"), String::from("transition:generic")]),
                    ..Default::default()
                }
            )
            .await?
    );

    // Click the URL and sign in,
    // then copy and paste the URL like “http://127.0.0.1/?iss=...&code=...” after it is redirected.

    print!("Redirected url: ");
    stdout().lock().flush()?;
    let mut url = String::new();
    stdin().lock().read_line(&mut url)?;

    let uri = url.trim().parse::<Uri>()?;
    let params = serde_html_form::from_str(uri.query().unwrap())?;
    let (session, _) = client.callback(params).await?;
    let agent = Agent::new(session);
    println!(
        "{:?}",
        agent
            .api
            .app
            .bsky
            .feed
            .get_timeline(
                atrium_api::app::bsky::feed::get_timeline::ParametersData {
                    algorithm: None,
                    cursor: None,
                    limit: 1.try_into().ok()
                }
                .into()
            )
            .await?
    );
    Ok(())
}
