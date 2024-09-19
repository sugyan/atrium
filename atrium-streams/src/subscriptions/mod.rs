pub mod frames;
pub mod handlers;

use std::{fmt::Debug, future::Future};

use futures::Stream;

/// A trait that defines the connection handler.
pub trait ConnectionHandler {
    /// The [`Self::HandledData`](ConnectionHandler::HandledData) type should be used to define the returned processed data type.
    type HandledData;
    /// The [`Self::HandlingError`](ConnectionHandler::HandlingError) type should be used to define the processing error type.
    type HandlingError: 'static + Send + Sync + Debug;

    /// Handles binary data coming from the connection. This function will deserialize the payload body and call the appropriate
    /// handler for each payload type.
    ///
    /// # Returns
    /// [`Result<Option<T>>`] like:
    /// - `Ok(Some(processedPayload))` where `processedPayload` is [`ProcessedPayload<ConnectionHandler::HandledData>`](ProcessedPayload)
    ///   if the payload was successfully processed.
    /// - `Ok(None)` if the payload was ignored.
    /// - `Err(e)` where `e` is [`ConnectionHandler::HandlingError`] if an error occurred while processing the payload.
    fn handle_payload(
        &self,
        t: String,
        payload: Vec<u8>,
    ) -> impl Future<Output = Result<Option<ProcessedPayload<Self::HandledData>>, Self::HandlingError>>;
}

/// A trait that defines a subscription.
/// It should be implemented by any struct that wants to handle a connection.
/// The `ConnectionPayload` type parameter is the type of the payload that will be received through the connection stream.
/// The `Error` type parameter is the type of the error that the specific subscription can return, following the lexicon.
pub trait Subscription<ConnectionPayload, Error: 'static + Send + Sync + Debug> {
    /// The `handle_connection` method should be implemented to handle the connection.
    ///
    /// # Returns
    /// A stream of processed payloads.
    fn handle_connection<H: ConnectionHandler + Sync>(
        connection: impl Stream<Item = ConnectionPayload> + Unpin,
        handler: H,
    ) -> impl Stream<Item = Result<ProcessedPayload<H::HandledData>, SubscriptionError<Error>>>;
}

/// This struct represents a processed payload.
/// It contains the sequence number (cursor) and the final processed data.
pub struct ProcessedPayload<Kind> {
    pub seq: Option<i64>, // Might be absent, like in the case of #info.
    pub data: Kind,
}

/// Helper function to convert between payload kinds.
impl<Kind> ProcessedPayload<Kind> {
    pub fn map<NewKind, F: FnOnce(Kind) -> NewKind>(self, f: F) -> ProcessedPayload<NewKind> {
        ProcessedPayload { seq: self.seq, data: f(self.data) }
    }
}

/// An error type that represents a subscription error.
///
/// `Abort` is a hard error, and the subscription should cancel.
/// This follows the [`ATProto Specs`](https://atproto.com/specs/event-stream).
///
/// `Unknown` is an error that is not recognized by the subscription.
/// This can be used to handle unexpected errors.
///
/// `Other` is an error specific to the subscription type.
/// This can be used to handle different kinds of errors, following the lexicon.
#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError<T> {
    #[error("Critical Subscription Error: {0}")]
    Abort(String),
    #[error("Unknown Subscription Error: {0}")]
    Unknown(String),
    #[error(transparent)]
    Other(T),
}
