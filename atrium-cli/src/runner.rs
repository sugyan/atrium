use crate::commands::Command;
use crate::store::SimpleJsonFileSessionStore;
use atrium_api::agent::{store::SessionStore, AtpAgent};
use atrium_api::types::string::{AtIdentifier, Datetime, Handle};
use atrium_api::xrpc::error::{Error, XrpcErrorKind};
use atrium_xrpc_client::reqwest::ReqwestClient;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncReadExt;

pub struct Runner {
    agent: AtpAgent<SimpleJsonFileSessionStore, ReqwestClient>,
    debug: bool,
    session_path: PathBuf,
    handle: Option<Handle>,
}

impl Runner {
    pub async fn new(pds_host: String, debug: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().unwrap();
        let dir = config_dir.join("atrium-cli");
        create_dir_all(&dir).await?;
        let session_path = dir.join("session.json");
        let store = SimpleJsonFileSessionStore::new(session_path.clone());
        let session = store.get_session().await;
        let handle = session.as_ref().map(|s| s.handle.clone());
        let agent = AtpAgent::new(ReqwestClient::new(pds_host), store);
        if let Some(s) = &session {
            agent.resume_session(s.clone()).await?;
        }
        Ok(Self {
            agent,
            debug,
            session_path,
            handle,
        })
    }
    pub async fn run(&self, command: Command) {
        let limit = 10.try_into().expect("within limit");
        match command {
            Command::Login(args) => {
                let result = self.agent.login(args.identifier, args.password).await;
                match result {
                    Ok(_) => {
                        println!("Login successful! Saved session to {:?}", self.session_path);
                    }
                    Err(err) => {
                        eprintln!("{err:#?}");
                    }
                }
            }
            Command::GetTimeline => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_timeline(atrium_api::app::bsky::feed::get_timeline::Parameters {
                            algorithm: None,
                            cursor: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetAuthorFeed(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_author_feed(atrium_api::app::bsky::feed::get_author_feed::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                            cursor: None,
                            filter: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetLikes(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_likes(atrium_api::app::bsky::feed::get_likes::Parameters {
                            cid: None,
                            cursor: None,
                            limit: Some(limit),
                            uri: args.uri.to_string(),
                        })
                        .await,
                );
            }
            Command::GetRepostedBy(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_reposted_by(atrium_api::app::bsky::feed::get_reposted_by::Parameters {
                            cid: None,
                            cursor: None,
                            limit: Some(limit),
                            uri: args.uri.to_string(),
                        })
                        .await,
                );
            }
            Command::GetActorFeeds(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_actor_feeds(atrium_api::app::bsky::feed::get_actor_feeds::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                            cursor: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetFeed(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_feed(atrium_api::app::bsky::feed::get_feed::Parameters {
                            cursor: None,
                            feed: args.uri.to_string(),
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetListFeed(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .feed
                        .get_list_feed(atrium_api::app::bsky::feed::get_list_feed::Parameters {
                            cursor: None,
                            limit: Some(limit),
                            list: args.uri.to_string(),
                        })
                        .await,
                );
            }
            Command::GetFollows(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .graph
                        .get_follows(atrium_api::app::bsky::graph::get_follows::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                            cursor: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetFollowers(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .graph
                        .get_followers(atrium_api::app::bsky::graph::get_followers::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                            cursor: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetLists(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .graph
                        .get_lists(atrium_api::app::bsky::graph::get_lists::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                            cursor: None,
                            limit: Some(limit),
                        })
                        .await,
                );
            }
            Command::GetList(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .graph
                        .get_list(atrium_api::app::bsky::graph::get_list::Parameters {
                            cursor: None,
                            limit: Some(limit),
                            list: args.uri.to_string(),
                        })
                        .await,
                );
            }
            Command::GetProfile(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .actor
                        .get_profile(atrium_api::app::bsky::actor::get_profile::Parameters {
                            actor: args
                                .actor
                                .or(self.handle.clone().map(AtIdentifier::Handle))
                                .unwrap(),
                        })
                        .await,
                );
            }
            Command::ListNotifications => {
                self.print(
                    &self
                        .agent
                        .api
                        .app
                        .bsky
                        .notification
                        .list_notifications(
                            atrium_api::app::bsky::notification::list_notifications::Parameters {
                                cursor: None,
                                limit: Some(limit),
                                seen_at: None,
                            },
                        )
                        .await,
                );
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
                        images.push(atrium_api::app::bsky::embed::images::Image {
                            alt: image
                                .file_name()
                                .map(OsStr::to_string_lossy)
                                .unwrap_or_default()
                                .into(),
                            aspect_ratio: None,
                            image: output.blob,
                        })
                    }
                }
                let embed = Some(
                    atrium_api::app::bsky::feed::post::RecordEmbedEnum::AppBskyEmbedImagesMain(
                        Box::new(atrium_api::app::bsky::embed::images::Main { images }),
                    ),
                );
                self.print(
                    &self
                        .agent
                        .api
                        .com
                        .atproto
                        .repo
                        .create_record(atrium_api::com::atproto::repo::create_record::Input {
                            collection: "app.bsky.feed.post".parse().expect("valid"),
                            record: atrium_api::records::Record::AppBskyFeedPost(Box::new(
                                atrium_api::app::bsky::feed::post::Record {
                                    created_at: Datetime::now(),
                                    embed,
                                    entities: None,
                                    facets: None,
                                    labels: None,
                                    langs: None,
                                    reply: None,
                                    tags: None,
                                    text: args.text,
                                },
                            )),
                            repo: self.handle.clone().unwrap().into(),
                            rkey: None,
                            swap_commit: None,
                            validate: None,
                        })
                        .await,
                );
            }
            Command::DeletePost(args) => {
                self.print(
                    &self
                        .agent
                        .api
                        .com
                        .atproto
                        .repo
                        .delete_record(atrium_api::com::atproto::repo::delete_record::Input {
                            collection: "app.bsky.feed.post".parse().expect("valid"),
                            repo: self.handle.clone().unwrap().into(),
                            rkey: args.uri.rkey,
                            swap_commit: None,
                            swap_record: None,
                        })
                        .await,
                );
            }
        }
    }
    fn print(
        &self,
        result: &Result<impl std::fmt::Debug + Serialize, Error<impl std::fmt::Debug>>,
    ) {
        match result {
            Ok(result) => {
                if self.debug {
                    println!("{:#?}", result);
                } else {
                    println!("{}", serde_json::to_string_pretty(result).unwrap());
                }
            }
            Err(err) => {
                if let Error::XrpcResponse(e) = err {
                    if let Some(XrpcErrorKind::Undefined(body)) = &e.error {
                        if e.status == 401 && body.error == Some("AuthMissing".into()) {
                            eprintln!("Login required. Use `atrium-cli login` to login.");
                            return;
                        }
                    }
                }
                eprintln!("{err:#?}");
            }
        }
    }
}
