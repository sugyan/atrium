//! This file defines the [`FrameHeader`] and [`Frame`] types, which are used to parse the payloads sent by the subscription through the event stream.
//! You can read more about the specs for these types in the [`ATProto documentation`](https://atproto.com/specs/event-stream)

#[cfg(test)]
mod tests;

use cbor4ii::core::utils::IoReader;
use ipld_core::ipld::Ipld;
use serde::Deserialize;
use serde_ipld_dagcbor::de::Deserializer;
use std::io::Cursor;

/// An error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Unknown frame type. Header: {0:?}")]
  UnknownFrameType(Ipld),
  #[error("Payload was empty. Header: {0:?}")]
  EmptyPayload(Ipld),
  #[error("Ipld Decoding error: {0}")]
  IpldDecoding(#[from] serde_ipld_dagcbor::DecodeError<std::io::Error>),
}

/// Represents the header of a frame. It's the first [`Ipld`] object in a Binary payload sent by a subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameHeader {
  Message { t: String },
  Error,
}

impl TryFrom<Ipld> for FrameHeader {
  type Error = self::Error;

  fn try_from(header: Ipld) -> Result<Self, <Self as TryFrom<Ipld>>::Error> {
    if let Ipld::Map(ref map) = header {
      if let Some(Ipld::Integer(i)) = map.get("op") {
        match i {
          1 => {
            if let Some(Ipld::String(s)) = map.get("t") {
              return Ok(Self::Message { t: s.to_owned() });
            }
          }
          -1 => return Ok(Self::Error),
          _ => {}
        }
      }
    }
    Err(Error::UnknownFrameType(header))
  }
}

/// Represents a frame sent by a subscription. It's the second [`Ipld`] object in a Binary payload sent by a subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
  Message {
    t: String,
    data: Vec<u8>,
  },
  Error {
    data: Vec<u8>,
  },
}

impl TryFrom<Vec<u8>> for Frame {
  type Error = self::Error;

  fn try_from(value: Vec<u8>) -> Result<Self, <Self as TryFrom<Vec<u8>>>::Error> {
    let mut cursor = Cursor::new(value);
    let mut deserializer = Deserializer::from_reader(IoReader::new(&mut cursor));
    let header = Deserialize::deserialize(&mut deserializer)?;

    // Error means the stream did not end (trailing data), which implies a second IPLD (in this case, the payload).
    // If the stream ended, the payload is empty, in which case we error.
    let data = if deserializer.end().is_err() {
      let pos = cursor.position() as usize;
      cursor.get_mut().drain(pos..).collect()
    } else {
      return Err(Error::EmptyPayload(header));
    };

    match FrameHeader::try_from(header)? {
      FrameHeader::Message { t } => Ok(Self::Message { t, data }),
      FrameHeader::Error => Ok(Self::Error { data }),
    }
  }
}
