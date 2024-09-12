pub mod firehose;
pub mod type_defs;

use std::marker::PhantomData;

use async_stream::stream;
use bon::bon;
use futures::{Stream, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use atrium_xrpc_wss::{atrium_api::com::atproto::sync::subscribe_repos, subscriptions::{
  frames::{self, Frame},
  ConnectionHandler, ProcessedPayload, Subscription, SubscriptionError,
}};

/// A struct that represents the repositories subscription, used in `com.atproto.sync.subscribeRepos`.
pub struct Repositories<ConnectionPayload> {
  /// This is only here to constrain the `ConnectionPayload` used in [`Subscription`], or else we get a compile error.
  _payload_kind: PhantomData<ConnectionPayload>,
}
#[bon]
impl<ConnectionPayload> Repositories<ConnectionPayload>
where
  Self: Subscription<ConnectionPayload, subscribe_repos::Error>,
{
  #[builder]
  /// Defines the builder for any generic `Repositories` struct that implements [`Subscription`].
  pub fn new<H: ConnectionHandler + Sync>(
    connection: impl Stream<Item = ConnectionPayload> + Unpin,
    handler: H,
  ) -> impl Stream<Item = Result<ProcessedPayload<H::HandledData>, SubscriptionError<subscribe_repos::Error>>>
  {
    Self::handle_connection(connection, handler)
  }
}

type WssResult = tokio_tungstenite::tungstenite::Result<Message>;
impl Subscription<WssResult, subscribe_repos::Error> for Repositories<WssResult> {
  fn handle_connection<H: ConnectionHandler + Sync>(
    mut connection: impl Stream<Item = WssResult> + Unpin,
    handler: H,
  ) -> impl Stream<Item = Result<ProcessedPayload<H::HandledData>, SubscriptionError<subscribe_repos::Error>>>
  {
    // Builds a new async stream that will deserialize the packets sent through the
    // TCP tunnel and then yield the results processed by the handler back to the caller.
    let stream = stream! {
      loop {
        match connection.next().await {
          None => break, // Server dropped connection
          Some(Err(e)) => { // WebSocket error
            // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
            //  and the client should drop the entire connection instead of skipping the frame."
            // https://atproto.com/specs/event-stream
            yield Err(SubscriptionError::Abort(format!("Received invalid frame. Error: {e:?}")));
            break;
          }
          Some(Ok(Message::Binary(data))) => {
            match Frame::try_from(data) {
              Ok(Frame::Message { t, data: payload }) => {
                match handler.handle_payload(t, payload).await {
                  Ok(Some(res)) => yield Ok(res), // Payload was successfully handled.
                  Ok(None) => {}, // Payload was ignored by Handler.
                  Err(e) => {
                    // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
                    //  and the client should drop the entire connection instead of skipping the frame."
                    // https://atproto.com/specs/event-stream
                    yield Err(SubscriptionError::Abort(format!("Received invalid payload. Error: {e:?}")));
                    break;
                  },
                }
              },
              Ok(Frame::Error { data }) => {
                yield match serde_ipld_dagcbor::from_reader::<subscribe_repos::Error, _>(data.as_slice()) {
                  Ok(e) => Err(SubscriptionError::Other(e)),
                  Err(e) => Err(SubscriptionError::Unknown(format!("Failed to decode error frame: {e:?}"))),
                };
                break;
              },
              Err(frames::Error::EmptyPayload(ipld)) => {
                // "Invalid framing or invalid DAG-CBOR encoding are hard frames::errors,
                //  and the client should drop the entire connection instead of skipping the frame."
                // https://atproto.com/specs/event-stream
                yield Err(SubscriptionError::Abort(format!("Received empty payload for header: {ipld:?}")));
                break;
              },
              Err(frames::Error::IpldDecoding(e)) => {
                // "Invalid framing or invalid DAG-CBOR encoding are hard errors,
                //  and the client should drop the entire connection instead of skipping the frame."
                // https://atproto.com/specs/event-stream
                yield Err(SubscriptionError::Abort(format!("Received invalid frame. Error: {e:?}")));
                break;
              },
              Err(frames::Error::UnknownFrameType(_)) => {
                // "Clients should ignore frames with headers that have unknown op or t values.
                //  Unknown fields in both headers and payloads should be ignored."
                // https://atproto.com/specs/event-stream
              },
            }
          }
          _ => {}, // Ignore other message types.
        }
      }
    };

    Box::pin(stream)
  }
}
