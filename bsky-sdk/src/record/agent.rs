use super::Record;
use crate::error::{Error, Result};
use crate::BskyAgent;
use atrium_api::agent::store::SessionStore;
use atrium_api::com::atproto::repo::create_record;
use atrium_api::records::KnownRecord;
use atrium_api::types::string::RecordKey;
use atrium_api::xrpc::XrpcClient;

pub enum CreateRecordSubject {
    AppBskyActorProfile(Box<atrium_api::app::bsky::actor::profile::Record>),
    AppBskyFeedGenerator(Box<atrium_api::app::bsky::feed::generator::Record>),
    AppBskyFeedLike(Box<atrium_api::app::bsky::feed::like::Record>),
    AppBskyFeedPost(Box<atrium_api::app::bsky::feed::post::Record>),
    AppBskyFeedRepost(Box<atrium_api::app::bsky::feed::repost::Record>),
    AppBskyFeedThreadgate(Box<atrium_api::app::bsky::feed::threadgate::Record>),
    AppBskyGraphBlock(Box<atrium_api::app::bsky::graph::block::Record>),
    AppBskyGraphFollow(Box<atrium_api::app::bsky::graph::follow::Record>),
    AppBskyGraphList(Box<atrium_api::app::bsky::graph::list::Record>),
    AppBskyGraphListblock(Box<atrium_api::app::bsky::graph::listblock::Record>),
    AppBskyGraphListitem(Box<atrium_api::app::bsky::graph::listitem::Record>),
    AppBskyGraphStarterpack(Box<atrium_api::app::bsky::graph::starterpack::Record>),
    AppBskyLabelerService(Box<atrium_api::app::bsky::labeler::service::Record>),
    ChatBskyActorDeclaration(Box<atrium_api::chat::bsky::actor::declaration::Record>),
}

impl TryFrom<atrium_api::records::Record> for CreateRecordSubject {
    type Error = ();

    fn try_from(record: atrium_api::records::Record) -> std::result::Result<Self, Self::Error> {
        match record {
            atrium_api::records::Record::Known(record) => Ok(record.into()),
            _ => Err(()),
        }
    }
}

impl From<KnownRecord> for CreateRecordSubject {
    fn from(value: KnownRecord) -> Self {
        match value {
            KnownRecord::AppBskyActorProfile(record) => Self::AppBskyActorProfile(record),
            KnownRecord::AppBskyFeedGenerator(record) => Self::AppBskyFeedGenerator(record),
            KnownRecord::AppBskyFeedLike(record) => Self::AppBskyFeedLike(record),
            KnownRecord::AppBskyFeedPost(record) => Self::AppBskyFeedPost(record),
            KnownRecord::AppBskyFeedRepost(record) => Self::AppBskyFeedRepost(record),
            KnownRecord::AppBskyFeedThreadgate(record) => Self::AppBskyFeedThreadgate(record),
            KnownRecord::AppBskyGraphBlock(record) => Self::AppBskyGraphBlock(record),
            KnownRecord::AppBskyGraphFollow(record) => Self::AppBskyGraphFollow(record),
            KnownRecord::AppBskyGraphList(record) => Self::AppBskyGraphList(record),
            KnownRecord::AppBskyGraphListblock(record) => Self::AppBskyGraphListblock(record),
            KnownRecord::AppBskyGraphListitem(record) => Self::AppBskyGraphListitem(record),
            KnownRecord::AppBskyGraphStarterpack(record) => Self::AppBskyGraphStarterpack(record),
            KnownRecord::AppBskyLabelerService(record) => Self::AppBskyLabelerService(record),
            KnownRecord::ChatBskyActorDeclaration(record) => Self::ChatBskyActorDeclaration(record),
        }
    }
}

macro_rules! into_create_record_subject {
    ($record:path, $record_data:path, $variant:ident) => {
        impl From<$record> for CreateRecordSubject {
            fn from(record: $record) -> Self {
                Self::$variant(Box::new(record))
            }
        }

        impl From<$record_data> for CreateRecordSubject {
            fn from(record_data: $record_data) -> Self {
                Self::$variant(Box::new(record_data.into()))
            }
        }
    };
}

into_create_record_subject!(
    atrium_api::app::bsky::actor::profile::Record,
    atrium_api::app::bsky::actor::profile::RecordData,
    AppBskyActorProfile
);
into_create_record_subject!(
    atrium_api::app::bsky::feed::generator::Record,
    atrium_api::app::bsky::feed::generator::RecordData,
    AppBskyFeedGenerator
);
into_create_record_subject!(
    atrium_api::app::bsky::feed::like::Record,
    atrium_api::app::bsky::feed::like::RecordData,
    AppBskyFeedLike
);
into_create_record_subject!(
    atrium_api::app::bsky::feed::post::Record,
    atrium_api::app::bsky::feed::post::RecordData,
    AppBskyFeedPost
);
into_create_record_subject!(
    atrium_api::app::bsky::feed::repost::Record,
    atrium_api::app::bsky::feed::repost::RecordData,
    AppBskyFeedRepost
);
into_create_record_subject!(
    atrium_api::app::bsky::feed::threadgate::Record,
    atrium_api::app::bsky::feed::threadgate::RecordData,
    AppBskyFeedThreadgate
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::block::Record,
    atrium_api::app::bsky::graph::block::RecordData,
    AppBskyGraphBlock
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::follow::Record,
    atrium_api::app::bsky::graph::follow::RecordData,
    AppBskyGraphFollow
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::list::Record,
    atrium_api::app::bsky::graph::list::RecordData,
    AppBskyGraphList
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::listblock::Record,
    atrium_api::app::bsky::graph::listblock::RecordData,
    AppBskyGraphListblock
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::listitem::Record,
    atrium_api::app::bsky::graph::listitem::RecordData,
    AppBskyGraphListitem
);
into_create_record_subject!(
    atrium_api::app::bsky::graph::starterpack::Record,
    atrium_api::app::bsky::graph::starterpack::RecordData,
    AppBskyGraphStarterpack
);
into_create_record_subject!(
    atrium_api::app::bsky::labeler::service::Record,
    atrium_api::app::bsky::labeler::service::RecordData,
    AppBskyLabelerService
);
into_create_record_subject!(
    atrium_api::chat::bsky::actor::declaration::Record,
    atrium_api::chat::bsky::actor::declaration::RecordData,
    ChatBskyActorDeclaration
);

impl<T, S> BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    /// Create a record with various types of data.
    /// For example, the Record families defined in [`KnownRecord`](atrium_api::records::KnownRecord) are supported.
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
        subject: impl Into<CreateRecordSubject>,
    ) -> Result<create_record::Output> {
        match subject.into() {
            CreateRecordSubject::AppBskyActorProfile(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyFeedGenerator(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyFeedLike(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyFeedPost(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyFeedRepost(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyFeedThreadgate(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphBlock(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphFollow(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphList(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphListblock(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphListitem(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyGraphStarterpack(record) => record.data.create(self).await,
            CreateRecordSubject::AppBskyLabelerService(record) => record.data.create(self).await,
            CreateRecordSubject::ChatBskyActorDeclaration(record) => record.data.create(self).await,
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
    pub async fn delete_record(&self, at_uri: impl AsRef<str>) -> Result<()> {
        let parts = at_uri
            .as_ref()
            .strip_prefix("at://")
            .ok_or(Error::InvalidAtUri)?
            .splitn(3, '/')
            .collect::<Vec<_>>();
        let repo = parts[0].parse().or(Err(Error::InvalidAtUri))?;
        let collection = parts[1].parse().or(Err(Error::InvalidAtUri))?;
        let rkey = parts[2]
            .parse::<RecordKey>()
            .or(Err(Error::InvalidAtUri))?
            .into();
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
