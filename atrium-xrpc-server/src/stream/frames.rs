use atrium_api::com::atproto::sync::subscribe_repos::{Commit, Message};
use libipld_core::ipld::Ipld;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameTypeError;

impl Display for FrameTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid frame type")
    }
}

impl Error for FrameTypeError {}

// original definition:
//```
// export enum FrameType {
//   Message = 1,
//   Error = -1,
// }
// export const messageFrameHeader = z.object({
//   op: z.literal(FrameType.Message), // Frame op
//   t: z.string().optional(), // Message body type discriminator
// })
// export type MessageFrameHeader = z.infer<typeof messageFrameHeader>
// export const errorFrameHeader = z.object({
//   op: z.literal(FrameType.Error),
// })
// export type ErrorFrameHeader = z.infer<typeof errorFrameHeader>
// ```
#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameHeader {
    Message(Option<String>),
    Error,
}

impl TryFrom<Ipld> for FrameHeader {
    type Error = FrameTypeError;

    fn try_from(value: Ipld) -> Result<Self, <FrameHeader as TryFrom<Ipld>>::Error> {
        if let Ipld::Map(map) = value {
            if let Some(Ipld::Integer(i)) = map.get("op") {
                match i {
                    1 => {
                        let t = if let Some(Ipld::String(s)) = map.get("t") {
                            Some(s.clone())
                        } else {
                            None
                        };
                        return Ok(FrameHeader::Message(t));
                    }
                    -1 => return Ok(FrameHeader::Error),
                    _ => {}
                }
            }
        }
        Err(FrameTypeError)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Message(Box<MessageFrame>),
    Error(ErrorFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageFrame {
    pub body: Message,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFrame {
    // TODO
    // body: Value,
}

impl TryFrom<&[u8]> for Frame {
    type Error = Box<dyn Error>;

    fn try_from(value: &[u8]) -> Result<Self, <Frame as TryFrom<&[u8]>>::Error> {
        let mut cursor = Cursor::new(value);
        let (left, right) = match serde_ipld_dagcbor::from_reader::<Ipld, _>(&mut cursor) {
            Err(serde_ipld_dagcbor::DecodeError::TrailingData) => {
                value.split_at(cursor.position() as usize)
            }
            _ => {
                // TODO
                return Err(Box::new(FrameTypeError));
            }
        };
        let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(left)?;
        let header = FrameHeader::try_from(ipld)?;
        match header {
            FrameHeader::Message(t) => match t.as_deref() {
                Some("#commit") => Ok(Frame::Message(Box::new(MessageFrame {
                    body: Message::Commit(Box::new(serde_ipld_dagcbor::from_slice::<Commit>(
                        right,
                    )?)),
                }))),
                _ => unimplemented!("{t:?}"),
            },
            FrameHeader::Error => Ok(Frame::Error(ErrorFrame {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialized_data(s: &str) -> Vec<u8> {
        assert!(s.len() % 2 == 0);
        let b2u = |b: u8| match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            _ => unreachable!(),
        };
        s.as_bytes()
            .chunks(2)
            .map(|b| (b2u(b[0]) << 4) + b2u(b[1]))
            .collect()
    }

    #[test]
    fn deserialize_message_frame_header() {
        // {"op": 1, "t": "#commit"}
        let data = serialized_data("a2626f700161746723636f6d6d6974");
        let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
        assert_eq!(
            FrameHeader::try_from(ipld),
            Ok(FrameHeader::Message(Some(String::from("#commit"))))
        );
    }

    #[test]
    fn deserialize_error_frame_header() {
        // {"op": -1}
        let data = serialized_data("a1626f7020");
        let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
        assert_eq!(FrameHeader::try_from(ipld), Ok(FrameHeader::Error));
    }

    #[test]
    fn deserialize_invalid_frame_header() {
        {
            // {"op": 2, "t": "#commit"}
            let data = serialized_data("a2626f700261746723636f6d6d6974");
            let ipld =
                serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
            assert_eq!(FrameHeader::try_from(ipld), Err(FrameTypeError));
        }
        {
            // {"op": -2}
            let data = serialized_data("a1626f7021");
            let ipld =
                serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
            assert_eq!(FrameHeader::try_from(ipld), Err(FrameTypeError));
        }
    }
}
