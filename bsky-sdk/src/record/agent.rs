use super::Record;
use crate::error::{Error, Result};
use crate::BskyAgent;
use atrium_api::agent::store::SessionStore;
use atrium_api::com::atproto::repo::{create_record, delete_record};
use atrium_api::record::KnownRecord;
use atrium_api::types::string::RecordKey;
use atrium_api::xrpc::XrpcClient;

impl<T, S> BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    /// Create a record with various types of data.
    /// For example, the Record families defined in [`KnownRecord`](atrium_api::record::KnownRecord) are supported.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bsky_sdk::{BskyAgent, Result};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let agent = BskyAgent::builder().build().await?;
    ///     let output = agent.create_record(atrium_api::app::bsky::graph::block::RecordData {
    ///         created_at: atrium_api::types::string::Datetime::now(),
    ///         subject: "did:fake:handle.test".parse().expect("invalid did"),
    ///     }).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_record(
        &self,
        subject: impl Into<KnownRecord>,
    ) -> Result<create_record::Output> {
        match subject.into() {
            KnownRecord::AppBskyActorProfile(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedGenerator(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedLike(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedPost(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedPostgate(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedRepost(record) => record.data.create(self).await,
            KnownRecord::AppBskyFeedThreadgate(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphBlock(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphFollow(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphList(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphListblock(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphListitem(record) => record.data.create(self).await,
            KnownRecord::AppBskyGraphStarterpack(record) => record.data.create(self).await,
            KnownRecord::AppBskyLabelerService(record) => record.data.create(self).await,
            KnownRecord::ChatBskyActorDeclaration(record) => record.data.create(self).await,
            KnownRecord::ComAtprotoLexiconSchema(record) => record.data.create(self).await,
        }
    }
    /// Delete a record with AT URI.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::InvalidAtUri`] if the `at_uri` is invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bsky_sdk::{BskyAgent, Result};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let agent = BskyAgent::builder().build().await?;
    ///     agent.delete_record("at://did:fake:handle.test/app.bsky.graph.block/3kxmfwtgfxl2w").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete_record(&self, at_uri: impl AsRef<str>) -> Result<delete_record::Output> {
        let parts = at_uri
            .as_ref()
            .strip_prefix("at://")
            .ok_or(Error::InvalidAtUri)?
            .splitn(3, '/')
            .collect::<Vec<_>>();
        let repo = parts[0].parse().or(Err(Error::InvalidAtUri))?;
        let collection = parts[1].parse().or(Err(Error::InvalidAtUri))?;
        let rkey = parts[2].parse::<RecordKey>().or(Err(Error::InvalidAtUri))?.into();
        Ok(self
            .api
            .com
            .atproto
            .repo
            .delete_record(
                atrium_api::com::atproto::repo::delete_record::InputData {
                    collection,
                    repo,
                    rkey,
                    swap_commit: None,
                    swap_record: None,
                }
                .into(),
            )
            .await?)
    }
}
