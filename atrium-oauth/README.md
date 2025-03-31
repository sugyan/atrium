# ATrium OAuth: atproto flavoured OAuth client

Core library for implementing [atproto][ATPROTO] OAuth clients.

[ATPROTO]: https://atproto.com/ 'AT Protocol'

## Usage

### Configuration

```rust
use atrium_identity::{
    did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL},
    handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig, DnsTxtResolver},
};
use atrium_oauth::{
    store::{session::MemorySessionStore, state::MemoryStateStore},
    AtprotoLocalhostClientMetadata, DefaultHttpClient, KnownScope, OAuthClient, OAuthClientConfig,
    OAuthResolverConfig, Scope,
};
use std::{error::Error, sync::Arc};

struct SomeDnsTxtResolver;

impl DnsTxtResolver for SomeDnsTxtResolver {
    async fn resolve(
        &self,
        _: &str,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync + 'static>> {
        todo!()
    }
}

fn main() {
    let http_client = Arc::new(DefaultHttpClient::default());
    let config = OAuthClientConfig {
        client_metadata: AtprotoLocalhostClientMetadata {
            redirect_uris: Some(vec![String::from("http://127.0.0.1/callback")]),
            scopes: Some(vec![
                Scope::Known(KnownScope::Atproto),
                Scope::Known(KnownScope::TransitionGeneric),
            ]),
        },
        keys: None,
        resolver: OAuthResolverConfig {
            did_resolver: CommonDidResolver::new(CommonDidResolverConfig {
                plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
                http_client: Arc::clone(&http_client),
            }),
            handle_resolver: AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
                dns_txt_resolver: SomeDnsTxtResolver,
                http_client: Arc::clone(&http_client),
            }),
            authorization_server_metadata: Default::default(),
            protected_resource_metadata: Default::default(),
        },
        // A store for saving state data while the user is being redirected to the authorization server.
        state_store: MemoryStateStore::default(),
        // A store for saving session data.
        session_store: MemorySessionStore::default(),
    };
    let Ok(client) = OAuthClient::new(config) else {
        panic!("failed to create oauth client");
    };
}
```

### Authentication

```rust,ignore
use atrium_oauth::{AuthorizeOptions, KnownScope, OAuthClient, Scope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OAuthClient::new(...)?;
    let url = client
        .authorize(
            "foo.bsky.team",
            AuthorizeOptions {
                scopes: vec![
                    Scope::Known(KnownScope::Atproto),
                    Scope::Known(KnownScope::TransitionGeneric),
                ],
                ..Default::default()
            },
        )
        .await?;

    ...

    Ok(())
}
```

Make user visit `url`. Then, once it was redirected to the callback URI, perform the following:

```rust,ignore
use atrium_api::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    ...

    let query_params = "code=...&state=...";
    let params = serde_html_form::from_str(query_params)?;
    let (oauth_session, _) = client.callback(params).await?;

    ...

    Ok(())
}
```

The sign-in process results in an [`OAuthSession`] instance that can be used to make
authenticated requests to the resource server. This instance will automatically
refresh the credentials when needed.

### Making authenticated requests

The [`atrium_oauth`](crate) package provides a [`OAuthSession`] class that can be
used to make authenticated requests to Bluesky's AppView. This can be achieved
by constructing an [`Agent`](atrium_api::agent::Agent) instance using the
[`OAuthSession`] instance.

```rust,ignore
use atrium_api::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    ...

    let agent = Agent::new(oauth_session);
    let output = agent
        .api
        .app
        .bsky
        .feed
        .get_timeline(
            atrium_api::app::bsky::feed::get_timeline::ParametersData {
                algorithm: None,
                cursor: None,
                limit: 3.try_into().ok(),
            }
            .into(),
        )
        .await?;
    for feed in &output.feed {
        println!("{feed:?}");
    }

    ...

    Ok(())
}
```
