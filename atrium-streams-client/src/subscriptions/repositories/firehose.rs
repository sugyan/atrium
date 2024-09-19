use std::{collections::BTreeMap, io::Cursor};

use futures::io::Cursor as FutCursor;
use ipld_core::cid::Cid;

use super::type_defs::{self, Operation};
use atrium_streams::{
    atrium_api::{
        com::atproto::sync::subscribe_repos::{
            self, AccountData, CommitData, HandleData, IdentityData, InfoData, MigrateData,
            RepoOpData, TombstoneData,
        },
        record::KnownRecord,
        types::Object,
    },
    subscriptions::{
        handlers::repositories::{HandledData, Handler, ProcessedData},
        ConnectionHandler, ProcessedPayload,
    },
};

/// Errors for this crate
#[derive(Debug, thiserror::Error)]
pub enum HandlingError {
    #[error("CAR Decoding error: {0}")]
    CarDecoding(#[from] rs_car::CarDecodeError),
    #[error("IPLD Decoding error: {0}")]
    IpldDecoding(#[from] serde_ipld_dagcbor::DecodeError<std::io::Error>),
}

#[derive(bon::Builder)]
pub struct Firehose {
    #[builder(default)]
    enable_commit: bool,
    #[builder(default)]
    enable_identity: bool,
    #[builder(default)]
    enable_account: bool,
    #[builder(default)]
    enable_handle: bool,
    #[builder(default)]
    enable_migrate: bool,
    #[builder(default)]
    enable_tombstone: bool,
    #[builder(default)]
    enable_info: bool,
}
impl ConnectionHandler for Firehose {
    type HandledData = HandledData<Self>;
    type HandlingError = self::HandlingError;

    async fn handle_payload(
        &self,
        t: String,
        payload: Vec<u8>,
    ) -> Result<Option<ProcessedPayload<Self::HandledData>>, Self::HandlingError> {
        let res = match t.as_str() {
            "#commit" => {
                if self.enable_commit {
                    self.process_commit(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Commit))
                } else {
                    None
                }
            }
            "#identity" => {
                if self.enable_identity {
                    self.process_identity(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Identity))
                } else {
                    None
                }
            }
            "#account" => {
                if self.enable_account {
                    self.process_account(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Account))
                } else {
                    None
                }
            }
            "#handle" => {
                if self.enable_handle {
                    self.process_handle(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Handle))
                } else {
                    None
                }
            }
            "#migrate" => {
                if self.enable_migrate {
                    self.process_migrate(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Migrate))
                } else {
                    None
                }
            }
            "#tombstone" => {
                if self.enable_tombstone {
                    self.process_tombstone(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Tombstone))
                } else {
                    None
                }
            }
            "#info" => {
                if self.enable_info {
                    self.process_info(serde_ipld_dagcbor::from_reader(payload.as_slice())?)
                        .await?
                        .map(|data| data.map(ProcessedData::Info))
                } else {
                    None
                }
            }
            _ => {
                // "Clients should ignore frames with headers that have unknown op or t values.
                //  Unknown fields in both headers and payloads should be ignored."
                // https://atproto.com/specs/event-stream
                None
            }
        };

        Ok(res)
    }
}

impl Handler for Firehose {
    type ProcessedCommitData = type_defs::ProcessedCommitData;
    async fn process_commit(
        &self,
        payload: subscribe_repos::Commit,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedCommitData>>, Self::HandlingError> {
        let CommitData { blobs, blocks, commit, ops, repo, rev, seq, since, time, too_big, .. } =
            payload.data;

        // If it is too big, the blocks and ops are not sent, so we skip the processing.
        let ops_opt = if too_big {
            None
        } else {
            // We read all the blocks from the CAR file and store them in a map
            // so that we can look up the data for each operation by its CID.
            let mut cursor = FutCursor::new(blocks);
            let mut map = rs_car::car_read_all(&mut cursor, true)
                .await?
                .0
                .into_iter()
                .map(compat_cid)
                .collect::<BTreeMap<_, _>>();

            // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
            //  and the client should drop the entire connection instead of skipping the frame."
            // https://atproto.com/specs/event-stream
            Some(process_ops(ops, &mut map)?)
        };

        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedCommitData { ops: ops_opt, blobs, commit, repo, rev, since, time },
        }))
    }

    type ProcessedIdentityData = type_defs::ProcessedIdentityData;
    async fn process_identity(
        &self,
        payload: subscribe_repos::Identity,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedIdentityData>>, Self::HandlingError> {
        let IdentityData { did, handle, seq, time } = payload.data;
        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedIdentityData { did, handle, time },
        }))
    }

    type ProcessedAccountData = type_defs::ProcessedAccountData;
    async fn process_account(
        &self,
        payload: subscribe_repos::Account,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedAccountData>>, Self::HandlingError> {
        let AccountData { did, seq, time, active, status } = payload.data;
        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedAccountData { did, active, status, time },
        }))
    }

    type ProcessedHandleData = type_defs::ProcessedHandleData;
    async fn process_handle(
        &self,
        payload: subscribe_repos::Handle,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedHandleData>>, Self::HandlingError> {
        let HandleData { did, handle, seq, time } = payload.data;
        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedHandleData { did, handle, time },
        }))
    }

    type ProcessedMigrateData = type_defs::ProcessedMigrateData;
    async fn process_migrate(
        &self,
        payload: subscribe_repos::Migrate,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedMigrateData>>, Self::HandlingError> {
        let MigrateData { did, migrate_to, seq, time } = payload.data;
        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedMigrateData { did, migrate_to, time },
        }))
    }

    type ProcessedTombstoneData = type_defs::ProcessedTombstoneData;
    async fn process_tombstone(
        &self,
        payload: subscribe_repos::Tombstone,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedTombstoneData>>, Self::HandlingError> {
        let TombstoneData { did, seq, time } = payload.data;
        Ok(Some(ProcessedPayload {
            seq: Some(seq),
            data: Self::ProcessedTombstoneData { did, time },
        }))
    }

    type ProcessedInfoData = InfoData;
    async fn process_info(
        &self,
        payload: subscribe_repos::Info,
    ) -> Result<Option<ProcessedPayload<Self::ProcessedInfoData>>, Self::HandlingError> {
        Ok(Some(ProcessedPayload { seq: None, data: payload.data }))
    }
}

// Transmute is here because the version of the `rs_car` crate for `cid` is 0.10.1 whereas
// the `ilpd_core` crate is 0.11.1. Should work regardless, given that the Cid type's
// memory layout was not changed between the two versions. Temporary fix.
// TODO: Find a better way to fix the version compatibility issue.
fn compat_cid((cid, item): (rs_car::Cid, Vec<u8>)) -> (ipld_core::cid::Cid, Vec<u8>) {
    (unsafe { std::mem::transmute::<_, Cid>(cid) }, item)
}

fn process_ops(
    ops: Vec<Object<RepoOpData>>,
    map: &mut BTreeMap<Cid, Vec<u8>>,
) -> Result<Vec<Operation>, serde_ipld_dagcbor::DecodeError<std::io::Error>> {
    let mut processed_ops = Vec::with_capacity(ops.len());
    for op in ops {
        processed_ops.push(process_op(map, op)?);
    }
    Ok(processed_ops)
}

/// Processes a single operation.
fn process_op(
    map: &mut BTreeMap<Cid, Vec<u8>>,
    op: Object<RepoOpData>,
) -> Result<Operation, serde_ipld_dagcbor::DecodeError<std::io::Error>> {
    let RepoOpData { action, path, cid } = op.data;

    // Finds in the map the `Record` with the operation's CID and deserializes it.
    // If the item is not found, returns `None`.
    let record = match cid.as_ref().and_then(|c| map.get_mut(&c.0)) {
        Some(item) => Some(serde_ipld_dagcbor::from_reader::<KnownRecord, _>(Cursor::new(item))?),
        None => None,
    };

    Ok(Operation { action, path, record })
}
