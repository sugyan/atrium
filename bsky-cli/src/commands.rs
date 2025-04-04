use bsky_sdk::api::types::string::AtIdentifier;
use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub enum Command {
    /// Login (Create an authentication session).
    Login(LoginArgs),
    /// Get a view of an actor's home timeline.
    GetTimeline,
    /// Get a view of an actor's feed.
    GetAuthorFeed(ActorArgs),
    /// Get a list of likes for a given post.
    GetLikes(UriArgs),
    /// Get a list of reposts for a given post.
    GetRepostedBy(UriArgs),
    /// Get a list of feeds created by an actor.
    GetActorFeeds(ActorArgs),
    /// Get a view of a hydrated feed.
    GetFeed(UriArgs),
    /// Get a view of a specified list,
    GetListFeed(UriArgs),
    /// Get a list of who an actor follows.
    GetFollows(ActorArgs),
    /// Get a list of an actor's followers.
    GetFollowers(ActorArgs),
    /// Get a list of the list created by an actor.
    GetLists(ActorArgs),
    /// Get detailed info of a specified list.
    GetList(UriArgs),
    /// Get detailed profile view of an actor.
    GetProfile(ActorArgs),
    /// Get preferences of an actor.
    GetPreferences,
    /// Get a list of notifications.
    ListNotifications,
    /// Get a list of chat conversations.
    ListConvos,
    /// Send a message to a chat conversation.
    SendConvoMessage(SendConvoMessageArgs),
    /// Create a new post.
    CreatePost(CreatePostArgs),
    /// Delete a post.
    DeletePost(UriArgs),
}

#[derive(Parser, Debug)]
pub struct LoginArgs {
    /// Handle or other identifier supported by the server for the authenticating user.
    #[arg(short, long)]
    pub(crate) identifier: String,
    /// Password
    #[arg(short, long)]
    pub(crate) password: String,
}

#[derive(Parser, Debug)]
pub struct ActorArgs {
    /// Actor's handle or did
    #[arg(short, long, value_parser)]
    pub(crate) actor: Option<AtIdentifier>,
}

#[derive(Parser, Debug)]
pub struct UriArgs {
    /// Record's URI
    #[arg(short, long, value_parser)]
    pub(crate) uri: AtUri,
}

#[derive(Parser, Debug)]
pub struct SendConvoMessageArgs {
    /// Actor's handle or did
    #[arg(short, long, value_parser)]
    pub(crate) actor: AtIdentifier,
    /// Message text
    #[arg(short, long)]
    pub(crate) text: String,
}

#[derive(Parser, Debug)]
pub struct CreatePostArgs {
    /// Post text
    #[arg(short, long)]
    pub(crate) text: String,
    /// Images to embed
    #[arg(short, long)]
    pub(crate) images: Vec<PathBuf>,
    /// Alt-Text for images
    #[arg(short, long)]
    pub(crate) alt_text: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct AtUri {
    pub(crate) did: String,
    pub(crate) collection: String,
    pub(crate) rkey: String,
}

impl FromStr for AtUri {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .strip_prefix("at://did:plc:")
            .ok_or(r#"record uri must start with "at://did:plc:""#)?
            .splitn(3, '/')
            .collect::<Vec<_>>();
        Ok(Self {
            did: format!("did:plc:{}", parts[0]),
            collection: parts[1].to_string(),
            rkey: parts[2].to_string(),
        })
    }
}

impl std::fmt::Display for AtUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at://{}/{}/{}", self.did, self.collection, self.rkey)
    }
}
