//! This file defines the types used in the Firehose handler.

use atrium_streams::atrium_api::{
    record::KnownRecord,
    types::{
        string::{Datetime, Did, Handle},
        CidLink,
    },
};

// region: Commit
#[derive(Debug)]
pub struct ProcessedCommitData {
    pub repo: Did,
    pub commit: CidLink,
    // `ops` can be `None` if the commit is marked as `too_big`.
    pub ops: Option<Vec<Operation>>,
    pub blobs: Vec<CidLink>,
    pub rev: String,
    pub since: Option<String>,
    pub time: Datetime,
}
#[derive(Debug)]
pub struct Operation {
    pub action: String,
    pub path: String,
    pub record: Option<KnownRecord>,
}
// endregion: Commit

// region: Identity
#[derive(Debug)]
pub struct ProcessedIdentityData {
    pub did: Did,
    pub handle: Option<Handle>,
    pub time: Datetime,
}
// endregion: Identity

// region: Account
#[derive(Debug)]
pub struct ProcessedAccountData {
    pub did: Did,
    pub active: bool,
    pub status: Option<String>,
    pub time: Datetime,
}
// endregion: Account

// region: Handle
#[derive(Debug)]
pub struct ProcessedHandleData {
    pub did: Did,
    pub handle: Handle,
    pub time: Datetime,
}
// endregion: Handle

// region: Migrate
#[derive(Debug)]
pub struct ProcessedMigrateData {
    pub did: Did,
    pub migrate_to: Option<String>,
    pub time: Datetime,
}
// endregion: Migrate

// region: Tombstone
#[derive(Debug)]
pub struct ProcessedTombstoneData {
    pub did: Did,
    pub time: Datetime,
}
// endregion: Tombstone
