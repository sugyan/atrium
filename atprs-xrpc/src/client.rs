use chrono::Utc;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future};

#[derive(Debug, Serialize)]
pub struct CreateSessionInput {
    pub identifier: String,
    pub password: String,
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
pub struct CreateSessionOutput {
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
}

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub fn create_session(
        &self,
        input: CreateSessionInput,
    ) -> impl Future<Output = Result<Response, Error>> {
        reqwest::Client::new()
            .post(
                "https://bsky.social/xrpc/com.atproto.server.createSession
            ",
            )
            .json(&input)
            .send()
    }
    pub fn get_profile(
        &self,
        actor: String,
        token: String,
    ) -> impl Future<Output = Result<Response, Error>> {
        reqwest::Client::new()
            .get("https://bsky.social/xrpc/app.bsky.actor.getProfile")
            .bearer_auth(token)
            .query(&[("actor", actor)])
            .send()
    }
    pub fn create_record(
        &self,
        input: CreateRecordInput,
        token: String,
    ) -> impl Future<Output = Result<Response, Error>> {
        reqwest::Client::new()
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .bearer_auth(token)
            .json(&input)
            .send()
    }
}
