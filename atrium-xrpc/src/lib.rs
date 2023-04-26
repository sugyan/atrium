use async_trait::async_trait;
use atrium_api::xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Default)]
pub struct XrpcReqwestClient {
    client: Client,
    auth: Option<String>,
    host: String,
}

impl XrpcReqwestClient {
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }
    pub fn set_auth(&mut self, auth: String) {
        self.auth = Some(auth);
    }
}

#[async_trait]
impl HttpClient for XrpcReqwestClient {
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>> {
        let res = self.client.execute(req.try_into()?).await?;
        let mut builder = Response::builder().status(res.status());
        for (k, v) in res.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(res.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

impl XrpcClient for XrpcReqwestClient {
    fn host(&self) -> &str {
        &self.host
    }
    fn auth(&self) -> Option<&str> {
        self.auth.as_deref()
    }
}

// TODO: use macro?
impl atrium_api::app::bsky::actor::get_profile::GetProfile for XrpcReqwestClient {}
impl atrium_api::app::bsky::actor::get_profiles::GetProfiles for XrpcReqwestClient {}
impl atrium_api::app::bsky::actor::get_suggestions::GetSuggestions for XrpcReqwestClient {}
impl atrium_api::app::bsky::actor::search_actors::SearchActors for XrpcReqwestClient {}
impl atrium_api::app::bsky::actor::search_actors_typeahead::SearchActorsTypeahead
    for XrpcReqwestClient
{
}
impl atrium_api::app::bsky::feed::get_author_feed::GetAuthorFeed for XrpcReqwestClient {}
impl atrium_api::app::bsky::feed::get_likes::GetLikes for XrpcReqwestClient {}
impl atrium_api::app::bsky::feed::get_post_thread::GetPostThread for XrpcReqwestClient {}
impl atrium_api::app::bsky::feed::get_posts::GetPosts for XrpcReqwestClient {}
impl atrium_api::app::bsky::feed::get_reposted_by::GetRepostedBy for XrpcReqwestClient {}
impl atrium_api::app::bsky::feed::get_timeline::GetTimeline for XrpcReqwestClient {}
impl atrium_api::app::bsky::graph::get_followers::GetFollowers for XrpcReqwestClient {}
impl atrium_api::app::bsky::graph::get_follows::GetFollows for XrpcReqwestClient {}
impl atrium_api::app::bsky::graph::get_mutes::GetMutes for XrpcReqwestClient {}
impl atrium_api::app::bsky::graph::mute_actor::MuteActor for XrpcReqwestClient {}
impl atrium_api::app::bsky::graph::unmute_actor::UnmuteActor for XrpcReqwestClient {}
impl atrium_api::app::bsky::notification::get_unread_count::GetUnreadCount for XrpcReqwestClient {}
impl atrium_api::app::bsky::notification::list_notifications::ListNotifications
    for XrpcReqwestClient
{
}
impl atrium_api::app::bsky::notification::update_seen::UpdateSeen for XrpcReqwestClient {}
impl atrium_api::app::bsky::unspecced::get_popular::GetPopular for XrpcReqwestClient {}
impl atrium_api::com::atproto::admin::disable_invite_codes::DisableInviteCodes
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::get_invite_codes::GetInviteCodes for XrpcReqwestClient {}
impl atrium_api::com::atproto::admin::get_moderation_action::GetModerationAction
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::get_moderation_actions::GetModerationActions
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::get_moderation_report::GetModerationReport
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::get_moderation_reports::GetModerationReports
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::get_record::GetRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::admin::get_repo::GetRepo for XrpcReqwestClient {}
impl atrium_api::com::atproto::admin::resolve_moderation_reports::ResolveModerationReports
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::reverse_moderation_action::ReverseModerationAction
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::search_repos::SearchRepos for XrpcReqwestClient {}
impl atrium_api::com::atproto::admin::take_moderation_action::TakeModerationAction
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::update_account_email::UpdateAccountEmail
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::admin::update_account_handle::UpdateAccountHandle
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::identity::resolve_handle::ResolveHandle for XrpcReqwestClient {}
impl atrium_api::com::atproto::identity::update_handle::UpdateHandle for XrpcReqwestClient {}
impl atrium_api::com::atproto::label::query_labels::QueryLabels for XrpcReqwestClient {}
impl atrium_api::com::atproto::moderation::create_report::CreateReport for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::apply_writes::ApplyWrites for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::create_record::CreateRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::delete_record::DeleteRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::describe_repo::DescribeRepo for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::get_record::GetRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::list_records::ListRecords for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::put_record::PutRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::repo::upload_blob::UploadBlob for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::create_account::CreateAccount for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::create_app_password::CreateAppPassword
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::server::create_invite_code::CreateInviteCode for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::create_invite_codes::CreateInviteCodes
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::server::create_session::CreateSession for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::delete_account::DeleteAccount for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::delete_session::DeleteSession for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::describe_server::DescribeServer for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::get_account_invite_codes::GetAccountInviteCodes
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::server::get_session::GetSession for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::list_app_passwords::ListAppPasswords for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::refresh_session::RefreshSession for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::request_account_delete::RequestAccountDelete
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::server::request_password_reset::RequestPasswordReset
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::server::reset_password::ResetPassword for XrpcReqwestClient {}
impl atrium_api::com::atproto::server::revoke_app_password::RevokeAppPassword
    for XrpcReqwestClient
{
}
impl atrium_api::com::atproto::sync::get_blob::GetBlob for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_blocks::GetBlocks for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_checkout::GetCheckout for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_commit_path::GetCommitPath for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_head::GetHead for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_record::GetRecord for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::get_repo::GetRepo for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::list_blobs::ListBlobs for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::list_repos::ListRepos for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::notify_of_update::NotifyOfUpdate for XrpcReqwestClient {}
impl atrium_api::com::atproto::sync::request_crawl::RequestCrawl for XrpcReqwestClient {}
