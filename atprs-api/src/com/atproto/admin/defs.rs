// This file is generated by atprs-codegen. Do not edit.
//! Definitions for the `com.atproto.admin.defs` namespace.

// com.atproto.admin.defs#acknowledge
/// Moderation action type: Acknowledge. Indicates that the content was reviewed and not considered to violate PDS rules.
pub struct Acknowledge;

// com.atproto.admin.defs#actionReversal
pub struct ActionReversal {
    pub created_at: String,
    pub created_by: String,
    pub reason: String,
}

// com.atproto.admin.defs#actionType
pub struct ActionType;

// com.atproto.admin.defs#actionView
pub struct ActionView {
    pub action: ActionType,
    pub create_label_vals: Option<Vec<String>>,
    pub created_at: String,
    pub created_by: String,
    pub id: i32,
    pub negate_label_vals: Option<Vec<String>>,
    pub reason: String,
    pub resolved_report_ids: Vec<i32>,
    pub reversal: Option<ActionReversal>,
    // pub subject: ...,
    pub subject_blob_cids: Vec<String>,
}

// com.atproto.admin.defs#actionViewCurrent
pub struct ActionViewCurrent {
    pub action: ActionType,
    pub id: i32,
}

// com.atproto.admin.defs#actionViewDetail
pub struct ActionViewDetail {
    pub action: ActionType,
    pub create_label_vals: Option<Vec<String>>,
    pub created_at: String,
    pub created_by: String,
    pub id: i32,
    pub negate_label_vals: Option<Vec<String>>,
    pub reason: String,
    pub resolved_reports: Vec<ReportView>,
    pub reversal: Option<ActionReversal>,
    // pub subject: ...,
    pub subject_blobs: Vec<BlobView>,
}

// com.atproto.admin.defs#blobView
pub struct BlobView {
    pub cid: String,
    pub created_at: String,
    // pub details: ...,
    pub mime_type: String,
    pub moderation: Option<Moderation>,
    pub size: i32,
}

// com.atproto.admin.defs#flag
/// Moderation action type: Flag. Indicates that the content was reviewed and considered to violate PDS rules, but may still be served.
pub struct Flag;

// com.atproto.admin.defs#imageDetails
pub struct ImageDetails {
    pub height: i32,
    pub width: i32,
}

// com.atproto.admin.defs#moderation
pub struct Moderation {
    pub current_action: Option<ActionViewCurrent>,
}

// com.atproto.admin.defs#moderationDetail
pub struct ModerationDetail {
    pub actions: Vec<ActionView>,
    pub current_action: Option<ActionViewCurrent>,
    pub reports: Vec<ReportView>,
}

// com.atproto.admin.defs#recordView
pub struct RecordView {
    pub blob_cids: Vec<String>,
    pub cid: String,
    pub indexed_at: String,
    pub moderation: Moderation,
    pub repo: RepoView,
    pub uri: String,
    // pub value: ...,
}

// com.atproto.admin.defs#recordViewDetail
pub struct RecordViewDetail {
    pub blobs: Vec<BlobView>,
    pub cid: String,
    pub indexed_at: String,
    pub labels: Option<Vec<crate::com::atproto::label::defs::Label>>,
    pub moderation: ModerationDetail,
    pub repo: RepoView,
    pub uri: String,
    // pub value: ...,
}

// com.atproto.admin.defs#repoRef
pub struct RepoRef {
    pub did: String,
}

// com.atproto.admin.defs#repoView
pub struct RepoView {
    pub did: String,
    pub email: Option<String>,
    pub handle: String,
    pub indexed_at: String,
    pub invited_by: Option<crate::com::atproto::server::defs::InviteCode>,
    pub moderation: Moderation,
    // pub related_records: Vec<...>
}

// com.atproto.admin.defs#repoViewDetail
pub struct RepoViewDetail {
    pub did: String,
    pub email: Option<String>,
    pub handle: String,
    pub indexed_at: String,
    pub invited_by: Option<crate::com::atproto::server::defs::InviteCode>,
    pub invites: Option<Vec<crate::com::atproto::server::defs::InviteCode>>,
    pub labels: Option<Vec<crate::com::atproto::label::defs::Label>>,
    pub moderation: ModerationDetail,
    // pub related_records: Vec<...>
}

// com.atproto.admin.defs#reportView
pub struct ReportView {
    pub created_at: String,
    pub id: i32,
    pub reason: Option<String>,
    pub reason_type: crate::com::atproto::moderation::defs::ReasonType,
    pub reported_by: String,
    pub resolved_by_action_ids: Vec<i32>,
    // pub subject: ...,
}

// com.atproto.admin.defs#reportViewDetail
pub struct ReportViewDetail {
    pub created_at: String,
    pub id: i32,
    pub reason: Option<String>,
    pub reason_type: crate::com::atproto::moderation::defs::ReasonType,
    pub reported_by: String,
    pub resolved_by_actions: Vec<crate::com::atproto::admin::defs::ActionView>,
    // pub subject: ...,
}

// com.atproto.admin.defs#takedown
/// Moderation action type: Takedown. Indicates that content should not be served by the PDS.
pub struct Takedown;

// com.atproto.admin.defs#videoDetails
pub struct VideoDetails {
    pub height: i32,
    pub length: i32,
    pub width: i32,
}
