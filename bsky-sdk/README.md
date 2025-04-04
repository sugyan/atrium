# Bsky SDK: [ATrium](https://github.com/atrium-rs/atrium)-based SDK for Bluesky.

[![](https://img.shields.io/crates/v/bsky-sdk)](https://crates.io/crates/bsky-sdk)
[![](https://img.shields.io/docsrs/bsky-sdk)](https://docs.rs/bsky-sdk)
[![](https://img.shields.io/crates/l/bsky-sdk)](https://github.com/atrium-rs/atrium/blob/main/LICENSE)
[![Rust](https://github.com/atrium-rs/atrium/actions/workflows/bsky-sdk.yml/badge.svg?branch=main)](https://github.com/atrium-rs/atrium/actions/workflows/bsky-sdk.yml)

- ✔️ APIs for ATProto and Bluesky.
- ✔️ Session management (same as `atrium-api`'s [`AtpAgent`](https://docs.rs/atrium-api/latest/atrium_api/agent/struct.AtpAgent.html)).
- ✔️ Moderation APIs.
- ✔️ A RichText library.

## Usage

### Session management

Log into a server using these APIs. You'll need an active session for most methods.

```rust,no_run
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    let session = agent.login("alice@mail.com", "hunter2").await?;
    Ok(())
}
```

You can save the agent config (including the session) to a file and load it later:

```rust,no_run
use bsky_sdk::agent::config::FileStore;
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    agent.login("...", "...").await?;
    agent
        .to_config()
        .await
        .save(&FileStore::new("config.json"))
        .await?;
    Ok(())
}
```

```rust,no_run
use bsky_sdk::agent::config::{Config, FileStore};
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder()
        .config(Config::load(&FileStore::new("config.json")).await?)
        .build()
        .await?;
    let result = agent.api.com.atproto.server.get_session().await;
    assert!(result.is_ok());
    Ok(())
}
```

### Moderation

The moderation APIs have almost the same functionality as the official SDK ([@atproto/api](https://www.npmjs.com/package/@atproto/api#moderation)).

```rust,no_run
use bsky_sdk::moderation::decision::DecisionContext;
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    // log in...

    // First get the user's moderation prefs and their label definitions
    let preferences = agent.get_preferences(true).await?;
    let moderator = agent.moderator(&preferences).await?;

    // in feeds
    let output = agent
        .api
        .app
        .bsky
        .feed
        .get_timeline(
            atrium_api::app::bsky::feed::get_timeline::ParametersData {
                algorithm: None,
                cursor: None,
                limit: None,
            }
            .into(),
        )
        .await?;
    for feed_view_post in &output.feed {
        // We call the appropriate moderation function for the content
        let post_mod = moderator.moderate_post(&feed_view_post.post);
        // don't include in feeds?
        println!(
            "{:?} (filter: {})",
            feed_view_post.post.cid.as_ref(),
            post_mod.ui(DecisionContext::ContentList).filter()
        );
    }
    Ok(())
}
```

### Posts

Create a Bluesky post:

```rust,no_run
use atrium_api::types::string::Datetime;
use bsky_sdk::BskyAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    agent.login("...", "...").await?;

    agent
        .create_record(atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: "Hello world, from Rust!".to_string(),
        })
        .await?;
    Ok(())
}
```

### RichText

Creating a RichText object from a string:

```rust,no_run
use bsky_sdk::rich_text::RichText;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = RichText::new_with_detect_facets(
        "Hello @alice.com, check out this link: https://example.com",
    )
    .await?;

    let segments = rt.segments();
    assert_eq!(segments.len(), 4);
    assert!(segments[0].text == "Hello ");
    assert!(segments[1].text == "@alice.com" && segments[1].mention().is_some());
    assert!(segments[2].text == ", check out this link: ");
    assert!(segments[3].text == "https://example.com" && segments[3].link().is_some());

    let record_data = atrium_api::app::bsky::feed::post::RecordData {
        created_at: atrium_api::types::string::Datetime::now(),
        embed: None,
        entities: None,
        facets: rt.facets,
        labels: None,
        langs: None,
        reply: None,
        tags: None,
        text: rt.text,
    };
    println!("{:?}", record_data);
    Ok(())
}
```

Calculating string lengths:

```rust
use bsky_sdk::rich_text::RichText;

fn main() {
    let rt = RichText::new("👨‍👩‍👧‍👧", None);
    assert_eq!(rt.text.len(), 25);
    assert_eq!(rt.grapheme_len(), 1);
}
