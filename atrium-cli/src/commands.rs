use clap::Parser;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub enum Command {
    /// Login (Create an authentication session.)
    Login(LoginArgs),
    /// Get a view of the actor's home timeline.
    GetTimeline,
    /// Get a view of an actor's feed.
    GetAuthorFeed(ActorArgs),
    /// Get the list of likes.
    GetLikes(UriArgs),
    /// Get a list of reposts.
    GetRepostedBy(UriArgs),
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
    #[arg(short, long)]
    pub(crate) actor: Option<String>,
}

#[derive(Parser, Debug)]
pub struct UriArgs {
    /// Record's URI
    #[arg(short, long, value_parser)]
    pub(crate) uri: AtUri,
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
