use crate::commands::Command;
use anyhow::{Context, Result};
use api::agent::bluesky::{AtprotoServiceType, BSKY_CHAT_DID};
use api::types::string::{AtIdentifier, Datetime, Handle};
use api::types::LimitedNonZeroU8;
use bsky_sdk::agent::config::{Config, FileStore};
use bsky_sdk::api;
use bsky_sdk::BskyAgent;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncReadExt;

pub struct Runner {
    agent: BskyAgent,
    limit: LimitedNonZeroU8<100>,
    debug: bool,
    config_path: PathBuf,
}

impl Runner {
    pub async fn new(
        pds_host: String,
        limit: LimitedNonZeroU8<100>,
        debug: bool,
        is_login: bool,
    ) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .with_context(|| format!("No config dir: {:?}", dirs::config_dir()))?;
        let dir = config_dir.join("bsky-cli");
        create_dir_all(&dir).await?;
        let config_path = dir.join("config.json");

        let agent = if is_login {
            BskyAgent::builder()
                .config(Config { endpoint: pds_host, ..Default::default() })
                .build()
                .await?
        } else {
            let store = FileStore::new(&config_path);
            let agent = BskyAgent::builder()
                .config(Config::load(&store).await.with_context(|| "Not logged in")?)
                .build()
                .await?;
            agent.to_config().await.save(&store).await?;
            agent
        };
        Ok(Self { agent, limit, debug, config_path })
    }
    pub async fn run(&self, command: Command) -> Result<()> {
        let limit = self.limit;
        match command {
            Command::Login(args) => {
                self.agent.login(args.identifier, args.password).await?;
                // Set labelers from preferences
                let preferences = self.agent.get_preferences(true).await?;
                self.agent.configure_labelers_from_preferences(&preferences);
                // Save config to file
                self.agent.to_config().await.save(&FileStore::new(&self.config_path)).await?;
                println!("Login successful! Saved config to {:?}", self.config_path);
                Ok(())
            }
            Command::GetTimeline => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_timeline(
                        api::app::bsky::feed::get_timeline::ParametersData {
                            algorithm: None,
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetAuthorFeed(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_author_feed(
                        api::app::bsky::feed::get_author_feed::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                            cursor: None,
                            filter: None,
                            include_pins: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetLikes(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_likes(
                        api::app::bsky::feed::get_likes::ParametersData {
                            cid: None,
                            cursor: None,
                            limit: Some(limit),
                            uri: args.uri.to_string(),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetRepostedBy(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_reposted_by(
                        api::app::bsky::feed::get_reposted_by::ParametersData {
                            cid: None,
                            cursor: None,
                            limit: Some(limit),
                            uri: args.uri.to_string(),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetActorFeeds(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_actor_feeds(
                        api::app::bsky::feed::get_actor_feeds::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetFeed(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_feed(
                        api::app::bsky::feed::get_feed::ParametersData {
                            cursor: None,
                            feed: args.uri.to_string(),
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetListFeed(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_list_feed(
                        api::app::bsky::feed::get_list_feed::ParametersData {
                            cursor: None,
                            limit: Some(limit),
                            list: args.uri.to_string(),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetFollows(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .graph
                    .get_follows(
                        api::app::bsky::graph::get_follows::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetFollowers(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .graph
                    .get_followers(
                        api::app::bsky::graph::get_followers::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetLists(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .graph
                    .get_lists(
                        api::app::bsky::graph::get_lists::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetList(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .graph
                    .get_list(
                        api::app::bsky::graph::get_list::ParametersData {
                            cursor: None,
                            limit: Some(limit),
                            list: args.uri.to_string(),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetProfile(args) => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .actor
                    .get_profile(
                        api::app::bsky::actor::get_profile::ParametersData {
                            actor: args.actor.unwrap_or(self.handle().await?.into()),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::GetPreferences => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .actor
                    .get_preferences(
                        api::app::bsky::actor::get_preferences::ParametersData {}.into(),
                    )
                    .await?,
            ),
            Command::ListNotifications => self.print(
                &self
                    .agent
                    .api
                    .app
                    .bsky
                    .notification
                    .list_notifications(
                        api::app::bsky::notification::list_notifications::ParametersData {
                            cursor: None,
                            limit: Some(limit),
                            priority: None,
                            seen_at: None,
                            reasons: None,
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::ListConvos => self.print(
                &self
                    .agent
                    .api_with_proxy(
                        BSKY_CHAT_DID.parse().expect("valid DID"),
                        AtprotoServiceType::BskyChat,
                    )
                    .chat
                    .bsky
                    .convo
                    .list_convos(
                        api::chat::bsky::convo::list_convos::ParametersData {
                            cursor: None,
                            limit: Some(limit),
                        }
                        .into(),
                    )
                    .await?,
            ),
            Command::SendConvoMessage(args) => {
                let did = match args.actor {
                    AtIdentifier::Handle(handle) => {
                        self.agent
                            .api
                            .com
                            .atproto
                            .identity
                            .resolve_handle(
                                api::com::atproto::identity::resolve_handle::ParametersData {
                                    handle: handle.clone(),
                                }
                                .into(),
                            )
                            .await?
                            .data
                            .did
                    }
                    AtIdentifier::Did(did) => did,
                };
                let chat = &self
                    .agent
                    .api_with_proxy(
                        BSKY_CHAT_DID.parse().expect("valid DID"),
                        AtprotoServiceType::BskyChat,
                    )
                    .chat;
                let convo = chat
                    .bsky
                    .convo
                    .get_convo_for_members(
                        api::chat::bsky::convo::get_convo_for_members::ParametersData {
                            members: vec![did],
                        }
                        .into(),
                    )
                    .await?;
                self.print(
                    &chat
                        .bsky
                        .convo
                        .send_message(
                            api::chat::bsky::convo::send_message::InputData {
                                convo_id: convo.data.convo.data.id,
                                message: api::chat::bsky::convo::defs::MessageInputData {
                                    embed: None,
                                    facets: None,
                                    text: args.text,
                                }
                                .into(),
                            }
                            .into(),
                        )
                        .await?,
                )
            }
            Command::CreatePost(args) => {
                let mut images = Vec::new();
                for image in &args.images {
                    if let Ok(mut file) = File::open(image).await {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).await.expect("read image file");
                        let output = self
                            .agent
                            .api
                            .com
                            .atproto
                            .repo
                            .upload_blob(buf)
                            .await
                            .expect("upload blob");
                        images.push(
                            api::app::bsky::embed::images::ImageData {
                                alt: image
                                    .file_name()
                                    .map(OsStr::to_string_lossy)
                                    .unwrap_or_default()
                                    .into(),
                                aspect_ratio: None,
                                image: output.data.blob,
                            }
                            .into(),
                        )
                    }
                }
                let embed = Some(api::types::Union::Refs(
                    api::app::bsky::feed::post::RecordEmbedRefs::AppBskyEmbedImagesMain(Box::new(
                        api::app::bsky::embed::images::MainData { images }.into(),
                    )),
                ));
                self.print(
                    &self
                        .agent
                        .create_record(api::app::bsky::feed::post::RecordData {
                            created_at: Datetime::now(),
                            embed,
                            entities: None,
                            facets: None,
                            labels: None,
                            langs: None,
                            reply: None,
                            tags: None,
                            text: args.text,
                        })
                        .await?,
                )
            }
            Command::DeletePost(args) => self.print(
                &self
                    .agent
                    .api
                    .com
                    .atproto
                    .repo
                    .delete_record(
                        api::com::atproto::repo::delete_record::InputData {
                            collection: "app.bsky.feed.post".parse().expect("valid"),
                            repo: self.handle().await?.into(),
                            rkey: args.uri.rkey,
                            swap_commit: None,
                            swap_record: None,
                        }
                        .into(),
                    )
                    .await?,
            ),
        }
    }
    fn print<T: std::fmt::Debug + Serialize>(&self, result: &T) -> Result<()> {
        if self.debug {
            println!("{:#?}", result);
        } else {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        Ok(())
    }
    async fn handle(&self) -> Result<Handle> {
        Ok(self.agent.get_session().await.with_context(|| "Not logged in")?.data.handle)
    }
}
