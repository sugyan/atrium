use crate::rich_text::RichText;
use crate::BskyAgent;
use atrium_api::agent::store::SessionStore;
use atrium_api::app::bsky::embed::images;
use atrium_api::app::bsky::feed::post::{RecordData, RecordEmbedRefs, RecordLabelsRefs, ReplyRef};
use atrium_api::app::bsky::richtext::facet;
use atrium_api::com::atproto::label::defs::{SelfLabelData, SelfLabelsData};
use atrium_api::record::KnownRecord;
use atrium_api::types::string::{Datetime, Language};
use atrium_api::types::Union;
use atrium_api::xrpc::XrpcClient;
use futures::future;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]

pub enum BuilderError {
    #[error(transparent)]
    Sdk(#[from] crate::error::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse lang: {0}")]
    Lang(langtag::Error),
}

pub type Result<T> = core::result::Result<T, BuilderError>;

#[derive(Debug)]
pub struct RecordBuilder {
    created_at: Option<Datetime>,
    embed: Option<atrium_api::types::Union<RecordEmbedRefs>>,
    facets: Option<Vec<atrium_api::app::bsky::richtext::facet::Main>>,
    labels: Option<Vec<String>>,
    langs: Option<Vec<atrium_api::types::string::Language>>,
    reply: Option<ReplyRef>,
    tags: Option<Vec<String>>,
    text: String,
}

impl RecordBuilder {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            created_at: None,
            embed: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: text.as_ref().into(),
        }
    }
    pub fn created_at(mut self, created_at: Datetime) -> Self {
        self.created_at = Some(created_at);
        self
    }
    pub fn embed(mut self, embed: Union<RecordEmbedRefs>) -> Self {
        self.embed = Some(embed);
        self
    }
    pub fn facets(mut self, facets: Vec<facet::Main>) -> Self {
        if !facets.is_empty() {
            self.facets = Some(facets);
        }
        self
    }
    pub fn labels(mut self, labels: Vec<impl AsRef<str>>) -> Self {
        if !labels.is_empty() {
            self.labels = Some(labels.into_iter().map(|s| s.as_ref().into()).collect());
        }
        self
    }
    pub fn langs(mut self, langs: Vec<Language>) -> Self {
        if !langs.is_empty() {
            self.langs = Some(langs);
        }
        self
    }
    pub fn reply(mut self, reply: ReplyRef) -> Self {
        self.reply = Some(reply);
        self
    }
    pub fn tags(mut self, tags: &[impl AsRef<str>]) -> Self {
        self.tags = Some(tags.iter().map(|s| s.as_ref().into()).collect());
        self
    }
    pub fn build(self) -> KnownRecord {
        KnownRecord::AppBskyFeedPost(Box::new(
            RecordData {
                created_at: self.created_at.unwrap_or(Datetime::now()),
                embed: self.embed,
                entities: None,
                facets: self.facets,
                labels: self.labels.map(|v| {
                    Union::Refs(RecordLabelsRefs::ComAtprotoLabelDefsSelfLabels(Box::new(
                        SelfLabelsData {
                            values: v
                                .into_iter()
                                .map(|s| SelfLabelData { val: s }.into())
                                .collect(),
                        }
                        .into(),
                    )))
                }),
                langs: self.langs,
                reply: self.reply,
                tags: self.tags,
                text: self.text,
            }
            .into(),
        ))
    }
}

impl From<RecordBuilder> for KnownRecord {
    fn from(builder: RecordBuilder) -> Self {
        builder.build()
    }
}

#[derive(Debug)]
pub struct Builder {
    inner: RecordBuilder,
    auto_detect_facets: bool,
    embed: Option<Embed>,
    langs: Option<Vec<String>>,
}

impl Builder {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            inner: RecordBuilder::new(text),
            auto_detect_facets: true,
            embed: None,
            langs: None,
        }
    }
    pub fn auto_detect_facets(mut self, value: bool) -> Self {
        self.auto_detect_facets = value;
        self
    }
    pub fn created_at(mut self, created_at: Datetime) -> Self {
        self.inner = self.inner.created_at(created_at);
        self
    }
    pub fn embed_images(mut self, images: Vec<impl Into<ImageSubject>>) -> Self {
        self.embed = Some(Embed::Images(
            images.into_iter().map(|val| val.into()).collect(),
        ));
        self
    }
    // pub fn embed_external(mut self) -> Self {
    //     todo!()
    // }
    // pub fn embed_record(mut self) -> Self {
    //     todo!()
    // }
    // pub fn embed_record_with_media(mut self) -> Self {
    //     todo!()
    // }
    pub fn facets(mut self, facets: Vec<facet::Main>) -> Self {
        self.inner = self.inner.facets(facets);
        self.auto_detect_facets = false;
        self
    }
    pub fn labels(mut self, labels: Vec<impl AsRef<str>>) -> Self {
        self.inner = self.inner.labels(labels);
        self
    }
    pub fn langs(mut self, langs: Vec<impl AsRef<str>>) -> Self {
        self.langs = Some(langs.into_iter().map(|s| s.as_ref().into()).collect());
        self
    }
    // pub fn reply(mut self, reply: ReplyRef) -> Self {
    //     self.reply = Some(reply);
    //     self
    // }
    pub fn tags(mut self, tags: &[impl AsRef<str>]) -> Self {
        self.inner = self.inner.tags(tags);
        self
    }
    pub async fn build<T, S>(mut self, agent: &BskyAgent<T, S>) -> Result<KnownRecord>
    where
        T: XrpcClient + Send + Sync,
        S: SessionStore + Send + Sync,
    {
        if let Some(embed) = &self.embed {
            let refs = match embed {
                Embed::Images(image_subjects) => {
                    let agent = Arc::new(agent);
                    let mut handles = Vec::new();
                    for subject in image_subjects {
                        match subject {
                            // read file and upload blob
                            ImageSubject::Path((path, alt)) => {
                                let mut input = Vec::with_capacity(path.metadata()?.len() as usize);
                                File::open(path)?.read_to_end(&mut input)?;
                                let alt = alt.as_ref().map_or(
                                    path.file_name()
                                        .map(|s| s.to_string_lossy().into())
                                        .unwrap_or_default(),
                                    |s| s.clone(),
                                );
                                let agent = agent.clone();
                                handles.push(async move {
                                    agent.api.com.atproto.repo.upload_blob(input).await.map(
                                        |output| images::ImageData {
                                            alt,
                                            aspect_ratio: None,
                                            image: output.data.blob,
                                        },
                                    )
                                })
                            }
                            ImageSubject::Uri(_) => {
                                todo!()
                            }
                        }
                    }
                    let mut images = Vec::new();
                    for result in future::join_all(handles).await {
                        let image_data = result.map_err(|e| BuilderError::Sdk(e.into()))?;
                        images.push(image_data.into());
                    }
                    RecordEmbedRefs::AppBskyEmbedImagesMain(Box::new(
                        images::MainData { images }.into(),
                    ))
                }
            };
            self.inner = self.inner.embed(Union::Refs(refs));
        }
        if let Some(langs) = &self.langs {
            self.inner = self.inner.langs(
                langs
                    .iter()
                    .map(|s| s.parse().map_err(BuilderError::Lang))
                    .collect::<Result<_>>()?,
            );
        }
        if self.auto_detect_facets {
            if let Some(facets) = RichText::new_with_detect_facets(&self.inner.text)
                .await?
                .facets
            {
                self.inner = self.inner.facets(facets);
            }
        }
        Ok(self.inner.build())
    }
}

#[derive(Debug)]
enum Embed {
    Images(Vec<ImageSubject>),
    // External,
    // Record,
    // RecordWithMedia,
}

#[derive(Debug)]
pub enum ImageSubject {
    Path((std::path::PathBuf, Option<String>)),
    Uri((http::Uri, Option<String>)),
}

impl<T> From<T> for ImageSubject
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        Self::Path((std::path::PathBuf::from(value.as_ref()), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use atrium_api::types::{Blob, BlobRef, CidLink, TypedBlobRef};
    use atrium_api::xrpc::http::{Request, Response};
    use atrium_api::xrpc::types::Header;
    use atrium_api::xrpc::HttpClient;

    struct MockClient;

    #[async_trait]
    impl HttpClient for MockClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> core::result::Result<
            Response<Vec<u8>>,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        > {
            let body = match request.uri().path().strip_prefix("/xrpc/") {
                Some(atrium_api::com::atproto::repo::upload_blob::NSID) => r#"{
                  "blob": {
                    "$type": "blob",
                    "ref": {
                      "$link": "bafyreiclp443lavogvhj3d2ob2cxbfuscni2k5jk7bebjzg7khl3esabwq"
                    },
                    "mimeType": "image/png",
                    "size": 8493
                  }
                }"#
                .as_bytes()
                .to_vec(),
                _ => unreachable!(),
            };
            Ok(Response::builder()
                .header(Header::ContentType, "application/json")
                .body(body)?)
        }
    }

    #[async_trait]
    impl XrpcClient for MockClient {
        fn base_uri(&self) -> String {
            String::new()
        }
    }

    async fn agent() -> BskyAgent<MockClient> {
        BskyAgent::builder()
            .client(MockClient)
            .build()
            .await
            .expect("failed to build agent")
    }

    #[test]
    fn record_builder() {
        {
            let record = RecordBuilder::new(String::new()).build();
            assert!(matches!(record, KnownRecord::AppBskyFeedPost(_)));
        }
        {
            let now = Datetime::now();
            let record = RecordBuilder::new("foo").created_at(now.clone()).build();
            assert_eq!(
                record,
                KnownRecord::AppBskyFeedPost(Box::new(
                    RecordData {
                        created_at: now,
                        embed: None,
                        entities: None,
                        facets: None,
                        labels: None,
                        langs: None,
                        reply: None,
                        tags: None,
                        text: String::from("foo"),
                    }
                    .into()
                ))
            );
        }
        {
            let now = Datetime::now();
            let record = RecordBuilder::new("bar")
                .created_at(now.clone())
                .labels(vec!["baz"])
                .langs(vec![
                    "en".parse().expect("invalid lang"),
                    "ja".parse().expect("invalid lang"),
                ])
                .build();
            assert_eq!(
                record,
                KnownRecord::AppBskyFeedPost(Box::new(
                    RecordData {
                        created_at: now,
                        embed: None,
                        entities: None,
                        facets: None,
                        labels: Some(Union::Refs(
                            RecordLabelsRefs::ComAtprotoLabelDefsSelfLabels(Box::new(
                                SelfLabelsData {
                                    values: vec![SelfLabelData {
                                        val: String::from("baz")
                                    }
                                    .into()]
                                }
                                .into()
                            ))
                        )),
                        langs: Some(vec![
                            "en".parse().expect("invalid lang"),
                            "ja".parse().expect("invalid lang"),
                        ]),
                        reply: None,
                        tags: None,
                        text: String::from("bar"),
                    }
                    .into()
                ))
            );
        }
    }

    #[tokio::test]
    async fn builder_build() {
        let record = Builder::new(String::new())
            .build(&agent().await)
            .await
            .expect("failed to build record");
        assert!(matches!(record, KnownRecord::AppBskyFeedPost(_)));
    }

    #[tokio::test]
    async fn builder_auto_detect_facets() {
        let now = Datetime::now();
        let record = Builder::new("foo #bar https://example.com")
            .created_at(now.clone())
            .build(&agent().await)
            .await
            .expect("failed to build record");
        assert_eq!(
            record,
            KnownRecord::AppBskyFeedPost(Box::new(
                RecordData {
                    created_at: now,
                    embed: None,
                    entities: None,
                    facets: Some(vec![
                        facet::MainData {
                            features: vec![Union::Refs(facet::MainFeaturesItem::Link(Box::new(
                                facet::LinkData {
                                    uri: String::from("https://example.com")
                                }
                                .into()
                            )))],
                            index: facet::ByteSliceData {
                                byte_end: 28,
                                byte_start: 9
                            }
                            .into()
                        }
                        .into(),
                        facet::MainData {
                            features: vec![Union::Refs(facet::MainFeaturesItem::Tag(Box::new(
                                facet::TagData {
                                    tag: String::from("bar")
                                }
                                .into()
                            )))],
                            index: facet::ByteSliceData {
                                byte_end: 8,
                                byte_start: 4
                            }
                            .into()
                        }
                        .into()
                    ]),
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                    text: String::from("foo #bar https://example.com"),
                }
                .into()
            ))
        );
    }

    #[tokio::test]
    async fn builder_no_auto_detect_facets() {
        {
            let now = Datetime::now();
            let record = Builder::new("foo #bar https://example.com")
                .created_at(now.clone())
                .auto_detect_facets(false)
                .build(&agent().await)
                .await
                .expect("failed to build record");
            assert_eq!(
                record,
                KnownRecord::AppBskyFeedPost(Box::new(
                    RecordData {
                        created_at: now,
                        embed: None,
                        entities: None,
                        facets: None,
                        labels: None,
                        langs: None,
                        reply: None,
                        tags: None,
                        text: String::from("foo #bar https://example.com"),
                    }
                    .into()
                ))
            );
        }
        {
            let now = Datetime::now();
            let record = Builder::new("foo #bar https://example.com")
                .created_at(now.clone())
                .facets(Vec::new())
                .build(&agent().await)
                .await
                .expect("failed to build record");
            assert_eq!(
                record,
                KnownRecord::AppBskyFeedPost(Box::new(
                    RecordData {
                        created_at: now,
                        embed: None,
                        entities: None,
                        facets: None,
                        labels: None,
                        langs: None,
                        reply: None,
                        tags: None,
                        text: String::from("foo #bar https://example.com"),
                    }
                    .into()
                ))
            );
        }
    }

    #[tokio::test]
    async fn builder_embed() {
        let now = Datetime::now();
        let record = Builder::new("embed images")
            .created_at(now.clone())
            .embed_images(vec!["tests/data/images/dummy_600x400_ffffff_cccccc.png"])
            .build(&agent().await)
            .await
            .expect("failed to build record");
        assert_eq!(
            record,
            KnownRecord::AppBskyFeedPost(Box::new(
                RecordData {
                    created_at: now,
                    embed: Some(Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(
                        Box::new(
                            images::MainData {
                                images: vec![images::ImageData {
                                    alt: String::from("dummy_600x400_ffffff_cccccc.png"),
                                    aspect_ratio: None,
                                    image: BlobRef::Typed(TypedBlobRef::Blob(Blob {
                                        r#ref: CidLink("bafyreiclp443lavogvhj3d2ob2cxbfuscni2k5jk7bebjzg7khl3esabwq".parse().expect("invalid cid")),
                                        mime_type: String::from("image/png"),
                                        size: 8493,
                                    }))
                                }
                                .into()]
                            }
                            .into()
                        )
                    ))),
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                    text: String::from("embed images"),
                }
                .into()
            ))
        );
    }
}
