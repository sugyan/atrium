use crate::stream::frames::Frame;
use anyhow::Result;
use atrium_api::com::atproto::sync::subscribe_repos::Commit;
use std::future::Future;

#[trait_variant::make(HttpService: Send)]
pub trait Subscription {
    async fn next(&mut self) -> Option<Result<Frame, <Frame as TryFrom<&[u8]>>::Error>>;
}

pub trait CommitHandler {
    fn handle_commit(&self, commit: &Commit) -> impl Future<Output = Result<()>>;
}
