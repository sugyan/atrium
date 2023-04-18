use chrono::Utc;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionInput {
    pub identifier: String,
    pub password: String,
}

pub type CreateSessionOutput = AuthInfo;

#[derive(Debug, Deserialize)]
pub struct GetProfileOutput {
    pub did: String,
    pub handle: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    #[serde(rename = "followersCount")]
    pub followers_count: Option<u32>,
    #[serde(rename = "followsCount")]
    pub follows_count: Option<u32>,
    #[serde(rename = "postsCount")]
    pub posts_count: Option<u32>,
    #[serde(rename = "indexedAt")]
    pub indexed_at: Option<String>,
    pub viewer: Option<ViewerState>,
}

#[derive(Debug, Deserialize)]
pub struct ViewerState {
    pub muted: Option<bool>,
    pub following: Option<String>,
    #[serde(rename = "followedBy")]
    pub followed_by: Option<String>,
}

#[derive(Debug, Serialize)]
struct RecordFeedPost {
    #[serde(rename = "$type")]
    pub type_: String,
    pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug)]
pub enum Record {
    FeedPost(String),
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let created_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        match self {
            Record::FeedPost(text) => RecordFeedPost {
                type_: String::from("app.bsky.feed.post"),
                text: text.clone(),
                created_at: created_at.to_string(),
            },
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateRecordInput {
    pub repo: String,
    pub collection: String,
    pub record: Record,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordOutput {
    pub uri: String,
    pub cid: String,
}

#[derive(Debug, Default)]
pub struct Client {
    host: String,
    auth: Option<AuthInfo>,
}

impl Client {
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }
    pub fn set_auth(&mut self, auth: AuthInfo) {
        self.auth = Some(auth);
    }
    pub async fn create_session(
        &self,
        input: CreateSessionInput,
    ) -> Result<CreateSessionOutput, Error> {
        reqwest::Client::new()
            .post(format!(
                "{}/xrpc/com.atproto.server.createSession",
                self.host
            ))
            .json(&input)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    pub async fn get_profile(&self, actor: String) -> Result<GetProfileOutput, Error> {
        let mut builder =
            reqwest::Client::new().get(format!("{}/xrpc/app.bsky.actor.getProfile", self.host));
        if let Some(auth_info) = &self.auth {
            builder = builder.bearer_auth(&auth_info.access_jwt);
        }
        builder
            .query(&[("actor", actor)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    pub async fn create_record(
        &self,
        input: CreateRecordInput,
    ) -> Result<CreateRecordOutput, Error> {
        let mut builder = reqwest::Client::new()
            .post(format!("{}/xrpc/com.atproto.repo.createRecord", self.host));
        if let Some(auth_info) = &self.auth {
            builder = builder.bearer_auth(&auth_info.access_jwt)
        }
        builder
            .json(&input)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
