use atrium_api::{
    agent::{store::MemorySessionStore, AtpAgent},
    client::AtpServiceClient,
    types::{
        string::{Datetime, Did},
        Collection, TryIntoUnknown, Union,
    },
    xrpc::{
        http::{uri::Builder, Request, Response},
        types::AuthorizationToken,
        HttpClient, XrpcClient,
    },
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use serde::Serialize;
use std::{fs::File, io::Read, path::PathBuf, time::Duration};
use tokio::time;

const VIDEO_SERVICE: &str = "https://video.bsky.app";
const VIDEO_SERVICE_DID: &str = "did:web:video.bsky.app";
const UPLOAD_VIDEO_PATH: &str = "/xrpc/app.bsky.video.uploadVideo";

/// Simple program to upload videos by ATrium API agent.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Identifier of the login user.
    #[arg(short, long)]
    identifier: String,
    /// App password of the login user.
    #[arg(short, long)]
    password: String,
    /// Video file to upload.
    #[arg(long, value_name = "VIDEO FILE")]
    video: PathBuf,
}

#[derive(Serialize)]
struct UploadParams {
    did: Did,
    name: String,
}

struct VideoClient {
    token: String,
    params: Option<UploadParams>,
    inner: ReqwestClient,
}

impl VideoClient {
    fn new(token: String, params: Option<UploadParams>) -> Self {
        Self {
            token,
            params,
            inner: ReqwestClient::new(
                // Actually, `base_uri` returns `VIDEO_SERVICE`, so there is no need to specify this.
                "https://dummy.example.com",
            ),
        }
    }
}

impl HttpClient for VideoClient {
    async fn send_http(
        &self,
        mut request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let is_upload_video = request.uri().path() == UPLOAD_VIDEO_PATH;
        // Hack: Append query parameters
        if is_upload_video {
            if let Some(params) = &self.params {
                *request.uri_mut() = Builder::from(request.uri().clone())
                    .path_and_query(format!(
                        "{UPLOAD_VIDEO_PATH}?{}",
                        serde_html_form::to_string(params)?
                    ))
                    .build()?;
            }
        }
        let mut response = self.inner.send_http(request).await;
        // Hack: Formatting an incorrect response body
        if is_upload_video {
            if let Ok(res) = response.as_mut() {
                *res.body_mut() =
                    [b"{\"jobStatus\":".to_vec(), res.body().to_vec(), b"}".to_vec()].concat();
            }
        }
        response
    }
}

impl XrpcClient for VideoClient {
    fn base_uri(&self) -> String {
        VIDEO_SERVICE.to_string()
    }
    async fn authorization_token(&self, _: bool) -> Option<AuthorizationToken> {
        Some(AuthorizationToken::Bearer(self.token.clone()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Read video file
    let data = {
        let mut file = File::open(&args.video)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        buf
    };

    // Login
    println!("Logging in...");
    let agent =
        AtpAgent::new(ReqwestClient::new("https://bsky.social"), MemorySessionStore::default());
    let session = agent.login(&args.identifier, &args.password).await?;

    // Check upload limits
    println!("Checking upload limits...");
    let limits = {
        let service_auth = agent
            .api
            .com
            .atproto
            .server
            .get_service_auth(
                atrium_api::com::atproto::server::get_service_auth::ParametersData {
                    aud: VIDEO_SERVICE_DID.parse().expect("invalid DID"),
                    exp: None,
                    lxm: atrium_api::app::bsky::video::get_upload_limits::NSID.parse().ok(),
                }
                .into(),
            )
            .await?;
        let client = AtpServiceClient::new(VideoClient::new(service_auth.data.token, None));
        client.service.app.bsky.video.get_upload_limits().await?
    };
    println!("{:?}", limits.data);
    if !limits.can_upload
        || limits.remaining_daily_bytes.map_or(false, |remain| remain < data.len() as i64)
        || limits.remaining_daily_videos.map_or(false, |remain| remain <= 0)
    {
        eprintln!("You cannot upload a video: {:?}", limits.data);
        return Ok(());
    }

    // Upload video
    println!("Uploading video...");
    let output = {
        let service_auth = agent
            .api
            .com
            .atproto
            .server
            .get_service_auth(
                atrium_api::com::atproto::server::get_service_auth::ParametersData {
                    aud: format!(
                        "did:web:{}",
                        agent.get_endpoint().await.strip_prefix("https://").unwrap()
                    )
                    .parse()
                    .expect("invalid DID"),
                    exp: None,
                    lxm: atrium_api::com::atproto::repo::upload_blob::NSID.parse().ok(),
                }
                .into(),
            )
            .await?;

        let filename = args
            .video
            .file_name()
            .and_then(|s| s.to_os_string().into_string().ok())
            .expect("failed to get filename");
        let client = AtpServiceClient::new(VideoClient::new(
            service_auth.data.token,
            Some(UploadParams { did: session.did.clone(), name: filename }),
        ));
        client.service.app.bsky.video.upload_video(data).await?
    };
    println!("{:?}", output.job_status.data);

    // Wait for the video to be uploaded
    let client = AtpServiceClient::new(ReqwestClient::new(VIDEO_SERVICE));
    let mut status = output.data.job_status.data;
    loop {
        status = client
            .service
            .app
            .bsky
            .video
            .get_job_status(
                atrium_api::app::bsky::video::get_job_status::ParametersData {
                    job_id: status.job_id.clone(),
                }
                .into(),
            )
            .await?
            .data
            .job_status
            .data;
        println!("{status:?}");
        if status.blob.is_some()
            || status.state == "JOB_STATE_CREATED"
            || status.state == "JOB_STATE_FAILED"
        {
            break;
        }
        time::sleep(Duration::from_millis(100)).await;
    }
    let Some(video) = status.blob else {
        eprintln!("Failed to get blob: {status:?}");
        return Ok(());
    };
    if let Some(message) = status.message {
        println!("{message}");
    }

    // Post to feed with the video
    println!("Video uploaded: {video:?}");
    let record = atrium_api::app::bsky::feed::post::RecordData {
        created_at: Datetime::now(),
        embed: Some(Union::Refs(
            atrium_api::app::bsky::feed::post::RecordEmbedRefs::AppBskyEmbedVideoMain(Box::new(
                atrium_api::app::bsky::embed::video::MainData {
                    alt: Some(String::from("alt text")),
                    aspect_ratio: None,
                    captions: None,
                    video,
                }
                .into(),
            )),
        )),
        entities: None,
        facets: None,
        labels: None,
        langs: None,
        reply: None,
        tags: None,
        text: String::new(),
    }
    .try_into_unknown()
    .expect("failed to convert record");
    let output = agent
        .api
        .com
        .atproto
        .repo
        .create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: atrium_api::app::bsky::feed::Post::nsid(),
                record,
                repo: session.data.did.into(),
                rkey: None,
                swap_commit: None,
                validate: Some(true),
            }
            .into(),
        )
        .await?;
    println!("{:?}", output.data);

    Ok(())
}
