// This file is generated by atrium-codegen. DO NOT EDIT.
//!A collection of ATP repository record types.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum Record {
    #[serde(rename = "app.bsky.actor.profile")]
    AppBskyActorProfile(Box<crate::app::bsky::actor::profile::Record>),
    #[serde(rename = "app.bsky.feed.generator")]
    AppBskyFeedGenerator(Box<crate::app::bsky::feed::generator::Record>),
    #[serde(rename = "app.bsky.feed.like")]
    AppBskyFeedLike(Box<crate::app::bsky::feed::like::Record>),
    #[serde(rename = "app.bsky.feed.post")]
    AppBskyFeedPost(Box<crate::app::bsky::feed::post::Record>),
    #[serde(rename = "app.bsky.feed.repost")]
    AppBskyFeedRepost(Box<crate::app::bsky::feed::repost::Record>),
    #[serde(rename = "app.bsky.feed.threadgate")]
    AppBskyFeedThreadgate(Box<crate::app::bsky::feed::threadgate::Record>),
    #[serde(rename = "app.bsky.graph.block")]
    AppBskyGraphBlock(Box<crate::app::bsky::graph::block::Record>),
    #[serde(rename = "app.bsky.graph.follow")]
    AppBskyGraphFollow(Box<crate::app::bsky::graph::follow::Record>),
    #[serde(rename = "app.bsky.graph.list")]
    AppBskyGraphList(Box<crate::app::bsky::graph::list::Record>),
    #[serde(rename = "app.bsky.graph.listblock")]
    AppBskyGraphListblock(Box<crate::app::bsky::graph::listblock::Record>),
    #[serde(rename = "app.bsky.graph.listitem")]
    AppBskyGraphListitem(Box<crate::app::bsky::graph::listitem::Record>),
}
