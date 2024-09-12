#![expect(unused_variables)]

use std::future::Future;

use atrium_api::com::atproto::sync::subscribe_repos;

use super::{ConnectionHandler, ProcessedPayload};

/// This type should be used to define [`ConnectionHandler::HandledData`](ConnectionHandler::HandledData)
/// for the `com.atproto.sync.subscribeRepos` subscription type.
pub type HandledData<H> = ProcessedData<
  <H as Handler>::ProcessedCommitData,
  <H as Handler>::ProcessedIdentityData,
  <H as Handler>::ProcessedAccountData,
  <H as Handler>::ProcessedHandleData,
  <H as Handler>::ProcessedMigrateData,
  <H as Handler>::ProcessedTombstoneData,
  <H as Handler>::ProcessedInfoData,
>;

/// Wrapper around all the possible types of processed data.
#[derive(Debug)]
pub enum ProcessedData<C, I0, A, H, M, T, I1> {
  Commit(C),
  Identity(I0),
  Account(A),
  Handle(H),
  Migrate(M),
  Tombstone(T),
  Info(I1),
}

/// A trait that defines a [`ConnectionHandler`] specific to the
/// `com.atproto.sync.subscribeRepos` subscription type.
///
/// Any struct that fully and correctly implements this trait will be able to
/// handle all the different payload types that the subscription can send.
/// Since the final desired result data type might change for each case, the
/// trait is generic, and the implementor must define the data type for each
/// payload they pretend to use. The same goes for the implementations of
/// each processing method, as the algorithm may vary.
pub trait Handler: ConnectionHandler {
  type ProcessedCommitData;
  /// Processes a payload of type `#commit`.
  fn process_commit(
    &self,
    payload: subscribe_repos::Commit,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedCommitData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedIdentityData;
  /// Processes a payload of type `#identity`.
  fn process_identity(
    &self,
    payload: subscribe_repos::Identity,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedIdentityData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedAccountData;
  /// Processes a payload of type `#account`.
  fn process_account(
    &self,
    payload: subscribe_repos::Account,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedAccountData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedHandleData;
  /// Processes a payload of type `#handle`.
  fn process_handle(
    &self,
    payload: subscribe_repos::Handle,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedHandleData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedMigrateData;
  /// Processes a payload of type `#migrate`.
  fn process_migrate(
    &self,
    payload: subscribe_repos::Migrate,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedMigrateData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedTombstoneData;
  /// Processes a payload of type `#tombstone`.
  fn process_tombstone(
    &self,
    payload: subscribe_repos::Tombstone,
  ) -> impl Future<
    Output = Result<Option<ProcessedPayload<Self::ProcessedTombstoneData>>, Self::HandlingError>,
  > {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }

  type ProcessedInfoData;
  /// Processes a payload of type `#info`.
  fn process_info(
    &self,
    payload: subscribe_repos::Info,
  ) -> impl Future<Output = Result<Option<ProcessedPayload<Self::ProcessedInfoData>>, Self::HandlingError>>
  {
    // Default implementation always returns `None`, meaning the implementation decided to ignore the payload.
    async { Ok(None) }
  }
}
