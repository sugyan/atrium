//! Record operations.
mod agent;

use std::future::Future;

use crate::error::{Error, Result};
use crate::BskyAgent;
use atrium_api::agent::store::SessionStore;
use atrium_api::com::atproto::repo::{
    create_record, delete_record, get_record, list_records, put_record,
};
use atrium_api::types::{Collection, LimitedNonZeroU8, TryIntoUnknown};
use atrium_api::xrpc::XrpcClient;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait Record<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    fn list(
        agent: &BskyAgent<T, S>,
        cursor: Option<String>,
        limit: Option<LimitedNonZeroU8<100u8>>,
    ) -> impl Future<Output = Result<list_records::Output>>;
    fn get(
        agent: &BskyAgent<T, S>,
        rkey: String,
    ) -> impl Future<Output = Result<get_record::Output>>;
    fn put(
        self,
        agent: &BskyAgent<T, S>,
        rkey: String,
    ) -> impl Future<Output = Result<put_record::Output>>;
    fn create(self, agent: &BskyAgent<T, S>)
        -> impl Future<Output = Result<create_record::Output>>;
    fn delete(
        agent: &BskyAgent<T, S>,
        rkey: String,
    ) -> impl Future<Output = Result<delete_record::Output>>;
}

macro_rules! record_impl {
    ($collection:path, $record:path, $record_data:path) => {
        impl<T, S> Record<T, S> for $record
        where
            T: XrpcClient + Send + Sync,
            S: SessionStore + Send + Sync,
        {
            async fn list(
                agent: &BskyAgent<T, S>,
                cursor: Option<String>,
                limit: Option<LimitedNonZeroU8<100u8>>,
            ) -> Result<list_records::Output> {
                let session = agent.get_session().await.ok_or(Error::NotLoggedIn)?;
                Ok(agent
                    .api
                    .com
                    .atproto
                    .repo
                    .list_records(
                        atrium_api::com::atproto::repo::list_records::ParametersData {
                            collection: <$collection>::nsid(),
                            cursor,
                            limit,
                            repo: session.data.did.into(),
                            reverse: None,
                            rkey_end: None,
                            rkey_start: None,
                        }
                        .into(),
                    )
                    .await?)
            }
            async fn get(agent: &BskyAgent<T, S>, rkey: String) -> Result<get_record::Output> {
                let session = agent.get_session().await.ok_or(Error::NotLoggedIn)?;
                Ok(agent
                    .api
                    .com
                    .atproto
                    .repo
                    .get_record(
                        atrium_api::com::atproto::repo::get_record::ParametersData {
                            cid: None,
                            collection: <$collection>::nsid(),
                            repo: session.data.did.into(),
                            rkey,
                        }
                        .into(),
                    )
                    .await?)
            }
            async fn put(
                self,
                agent: &BskyAgent<T, S>,
                rkey: String,
            ) -> Result<put_record::Output> {
                let session = agent.get_session().await.ok_or(Error::NotLoggedIn)?;
                Ok(agent
                    .api
                    .com
                    .atproto
                    .repo
                    .put_record(
                        atrium_api::com::atproto::repo::put_record::InputData {
                            collection: <$collection>::nsid(),
                            record: self.try_into_unknown()?,
                            repo: session.data.did.into(),
                            rkey,
                            swap_commit: None,
                            swap_record: None,
                            validate: None,
                        }
                        .into(),
                    )
                    .await?)
            }
            async fn create(self, agent: &BskyAgent<T, S>) -> Result<create_record::Output> {
                let session = agent.get_session().await.ok_or(Error::NotLoggedIn)?;
                Ok(agent
                    .api
                    .com
                    .atproto
                    .repo
                    .create_record(
                        atrium_api::com::atproto::repo::create_record::InputData {
                            collection: <$collection>::nsid(),
                            record: self.try_into_unknown()?,
                            repo: session.data.did.into(),
                            rkey: None,
                            swap_commit: None,
                            validate: None,
                        }
                        .into(),
                    )
                    .await?)
            }
            async fn delete(
                agent: &BskyAgent<T, S>,
                rkey: String,
            ) -> Result<delete_record::Output> {
                let session = agent.get_session().await.ok_or(Error::NotLoggedIn)?;
                Ok(agent
                    .api
                    .com
                    .atproto
                    .repo
                    .delete_record(
                        atrium_api::com::atproto::repo::delete_record::InputData {
                            collection: <$collection>::nsid(),
                            repo: session.data.did.into(),
                            rkey,
                            swap_commit: None,
                            swap_record: None,
                        }
                        .into(),
                    )
                    .await?)
            }
        }

        impl<T, S> Record<T, S> for $record_data
        where
            T: XrpcClient + Send + Sync,
            S: SessionStore + Send + Sync,
        {
            async fn list(
                agent: &BskyAgent<T, S>,
                cursor: Option<String>,
                limit: Option<LimitedNonZeroU8<100u8>>,
            ) -> Result<list_records::Output> {
                <$record>::list(agent, cursor, limit).await
            }
            async fn get(agent: &BskyAgent<T, S>, rkey: String) -> Result<get_record::Output> {
                <$record>::get(agent, rkey).await
            }
            async fn put(
                self,
                agent: &BskyAgent<T, S>,
                rkey: String,
            ) -> Result<put_record::Output> {
                <$record>::from(self).put(agent, rkey).await
            }
            async fn create(self, agent: &BskyAgent<T, S>) -> Result<create_record::Output> {
                <$record>::from(self).create(agent).await
            }
            async fn delete(
                agent: &BskyAgent<T, S>,
                rkey: String,
            ) -> Result<delete_record::Output> {
                <$record>::delete(agent, rkey).await
            }
        }
    };
}

record_impl!(
    atrium_api::com::atproto::lexicon::Schema,
    atrium_api::com::atproto::lexicon::schema::Record,
    atrium_api::com::atproto::lexicon::schema::RecordData
);
record_impl!(
    atrium_api::app::bsky::actor::Profile,
    atrium_api::app::bsky::actor::profile::Record,
    atrium_api::app::bsky::actor::profile::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Generator,
    atrium_api::app::bsky::feed::generator::Record,
    atrium_api::app::bsky::feed::generator::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Like,
    atrium_api::app::bsky::feed::like::Record,
    atrium_api::app::bsky::feed::like::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Post,
    atrium_api::app::bsky::feed::post::Record,
    atrium_api::app::bsky::feed::post::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Postgate,
    atrium_api::app::bsky::feed::postgate::Record,
    atrium_api::app::bsky::feed::postgate::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Repost,
    atrium_api::app::bsky::feed::repost::Record,
    atrium_api::app::bsky::feed::repost::RecordData
);
record_impl!(
    atrium_api::app::bsky::feed::Threadgate,
    atrium_api::app::bsky::feed::threadgate::Record,
    atrium_api::app::bsky::feed::threadgate::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::Block,
    atrium_api::app::bsky::graph::block::Record,
    atrium_api::app::bsky::graph::block::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::Follow,
    atrium_api::app::bsky::graph::follow::Record,
    atrium_api::app::bsky::graph::follow::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::List,
    atrium_api::app::bsky::graph::list::Record,
    atrium_api::app::bsky::graph::list::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::Listblock,
    atrium_api::app::bsky::graph::listblock::Record,
    atrium_api::app::bsky::graph::listblock::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::Listitem,
    atrium_api::app::bsky::graph::listitem::Record,
    atrium_api::app::bsky::graph::listitem::RecordData
);
record_impl!(
    atrium_api::app::bsky::graph::Starterpack,
    atrium_api::app::bsky::graph::starterpack::Record,
    atrium_api::app::bsky::graph::starterpack::RecordData
);
record_impl!(
    atrium_api::app::bsky::labeler::Service,
    atrium_api::app::bsky::labeler::service::Record,
    atrium_api::app::bsky::labeler::service::RecordData
);
record_impl!(
    atrium_api::chat::bsky::actor::Declaration,
    atrium_api::chat::bsky::actor::declaration::Record,
    atrium_api::chat::bsky::actor::declaration::RecordData
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::BskyAgentBuilder;
    use crate::tests::FAKE_CID;
    use atrium_api::agent::Session;
    use atrium_api::com::atproto::server::create_session::OutputData;
    use atrium_api::types::string::Datetime;
    use atrium_api::xrpc::http::{Request, Response};
    use atrium_api::xrpc::types::Header;
    use atrium_api::xrpc::{HttpClient, XrpcClient};

    struct MockClient;

    impl HttpClient for MockClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> core::result::Result<
            Response<Vec<u8>>,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        > {
            let body = match request.uri().path() {
                "/xrpc/com.atproto.repo.createRecord" => {
                    serde_json::to_vec(&create_record::OutputData {
                        cid: FAKE_CID.parse().expect("invalid cid"),
                        commit: None,
                        uri: String::from("at://did:fake:handle.test/app.bsky.feed.post/somerkey"),
                        validation_status: None,
                    })?
                }
                "/xrpc/com.atproto.repo.deleteRecord" => {
                    serde_json::to_vec(&delete_record::OutputData { commit: None })?
                }
                _ => unreachable!(),
            };
            Ok(Response::builder()
                .header(Header::ContentType, "application/json")
                .status(200)
                .body(body)?)
        }
    }

    impl XrpcClient for MockClient {
        fn base_uri(&self) -> String {
            String::new()
        }
    }

    struct MockSessionStore;

    impl SessionStore for MockSessionStore {
        async fn get_session(&self) -> Option<Session> {
            Some(
                OutputData {
                    access_jwt: String::from("access"),
                    active: None,
                    did: "did:fake:handle.test".parse().expect("invalid did"),
                    did_doc: None,
                    email: None,
                    email_auth_factor: None,
                    email_confirmed: None,
                    handle: "handle.test".parse().expect("invalid handle"),
                    refresh_jwt: String::from("refresh"),
                    status: None,
                }
                .into(),
            )
        }
        async fn set_session(&self, _: Session) {}
        async fn clear_session(&self) {}
    }

    #[tokio::test]
    async fn actor_profile() -> Result<()> {
        let agent = BskyAgentBuilder::new(MockClient).store(MockSessionStore).build().await?;
        // create
        let output = atrium_api::app::bsky::actor::profile::RecordData {
            avatar: None,
            banner: None,
            created_at: None,
            description: None,
            display_name: None,
            joined_via_starter_pack: None,
            labels: None,
            pinned_post: None,
        }
        .create(&agent)
        .await?;
        assert_eq!(
            output,
            create_record::OutputData {
                cid: FAKE_CID.parse().expect("invalid cid"),
                commit: None,
                uri: String::from("at://did:fake:handle.test/app.bsky.feed.post/somerkey"),
                validation_status: None,
            }
            .into()
        );
        // delete
        atrium_api::app::bsky::actor::profile::Record::delete(&agent, String::from("somerkey"))
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn feed_post() -> Result<()> {
        let agent = BskyAgentBuilder::new(MockClient).store(MockSessionStore).build().await?;
        // create
        let output = atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: String::from("text"),
        }
        .create(&agent)
        .await?;
        assert_eq!(
            output,
            create_record::OutputData {
                cid: FAKE_CID.parse().expect("invalid cid"),
                commit: None,
                uri: String::from("at://did:fake:handle.test/app.bsky.feed.post/somerkey"),
                validation_status: None,
            }
            .into()
        );
        // delete
        atrium_api::app::bsky::feed::post::Record::delete(&agent, String::from("somerkey")).await?;
        Ok(())
    }

    #[tokio::test]
    async fn graph_follow() -> Result<()> {
        let agent = BskyAgentBuilder::new(MockClient).store(MockSessionStore).build().await?;
        // create
        let output = atrium_api::app::bsky::graph::follow::RecordData {
            created_at: Datetime::now(),
            subject: "did:fake:handle.test".parse().expect("invalid did"),
        }
        .create(&agent)
        .await?;
        assert_eq!(
            output,
            create_record::OutputData {
                cid: FAKE_CID.parse().expect("invalid cid"),
                commit: None,
                uri: String::from("at://did:fake:handle.test/app.bsky.feed.post/somerkey"),
                validation_status: None,
            }
            .into()
        );
        // delete
        atrium_api::app::bsky::graph::follow::Record::delete(&agent, String::from("somerkey"))
            .await?;
        Ok(())
    }
}
