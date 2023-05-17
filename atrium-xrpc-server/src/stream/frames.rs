use atrium_api::com::atproto::sync::subscribe_repos::Commit;
use ciborium::Value;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameHeader {
    Message(Option<String>),
    Error,
}

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
impl TryFrom<Value> for FrameHeader {
    type Error = FrameTypeError;

    fn try_from(value: Value) -> Result<Self, <FrameHeader as TryFrom<Value>>::Error> {
        let (mut op, mut t) = (None, None);
        if let Some(map) = value.as_map() {
            for (k, v) in map {
                match (k.as_text(), v) {
                    (Some("op"), Value::Integer(i)) => match i8::try_from(*i) {
                        Ok(1) => op = Some(true),
                        Ok(-1) => op = Some(false),
                        _ => {}
                    },
                    (Some("t"), Value::Text(s)) => t = Some(s.clone()),
                    _ => {}
                }
            }
        }
        if let Some(b) = op {
            if b {
                Ok(FrameHeader::Message(t))
            } else {
                Ok(FrameHeader::Error)
            }
        } else {
            Err(FrameTypeError)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Message(MessageFrame),
    Error(ErrorFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageFrame {
    pub body: MessageEnum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorFrame {
    // TODO
    // body: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageEnum {
    Commit(Commit),
    // Handle(Handle),
    // Migrate(Migrate),
    // Tombstone(Tombstone),
    // Info(Info),
}

impl TryFrom<&[u8]> for Frame {
    type Error = Box<dyn Error>;

    fn try_from(value: &[u8]) -> Result<Self, <Frame as TryFrom<&[u8]>>::Error> {
        let mut cursor = Cursor::new(value);
        let value = ciborium::de::from_reader::<Value, _>(&mut cursor)?;
        let header = FrameHeader::try_from(value)?;
        match header {
            FrameHeader::Message(t) => match t.as_deref() {
                Some("#commit") => Ok(Frame::Message(MessageFrame {
                    body: MessageEnum::Commit(serde_ipld_dagcbor::from_reader::<Commit, _>(
                        &mut cursor,
                    )?),
                })),
                _ => unimplemented!("{t:?}"),
            },
            FrameHeader::Error => Ok(Frame::Error(ErrorFrame {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::Value;

    fn serialize_value(value: &Value) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(value, &mut buf).expect("failed to serialize");
        buf
    }

    #[test]
    fn deserialize_message_frame_header() {
        let data = serialize_value(&Value::Map(vec![
            (Value::Text("op".into()), Value::Integer(1.into())),
            (Value::Text("t".into()), Value::Text("#commit".into())),
        ]));
        let value =
            ciborium::de::from_reader::<Value, _>(data.as_slice()).expect("failed to deserialize");
        assert_eq!(
            FrameHeader::try_from(value),
            Ok(FrameHeader::Message(Some(String::from("#commit"))))
        );
    }

    #[test]
    fn deserialize_error_frame_header() {
        let data = serialize_value(&Value::Map(vec![(
            Value::Text("op".into()),
            Value::Integer((-1).into()),
        )]));
        let value =
            ciborium::de::from_reader::<Value, _>(data.as_slice()).expect("failed to deserialize");
        assert_eq!(FrameHeader::try_from(value), Ok(FrameHeader::Error));
    }

    #[test]
    fn deserialize_invalid_frame_header() {
        {
            let data = serialize_value(&Value::Map(vec![
                (Value::Text("op".into()), Value::Integer(2.into())),
                (Value::Text("t".into()), Value::Text("#commit".into())),
            ]));
            let value = ciborium::de::from_reader::<Value, _>(data.as_slice())
                .expect("failed to deserialize");
            assert_eq!(FrameHeader::try_from(value), Err(FrameTypeError));
        }
        {
            let data = serialize_value(&Value::Map(vec![(
                Value::Text("op".into()),
                Value::Integer((-2).into()),
            )]));
            let value = ciborium::de::from_reader::<Value, _>(data.as_slice())
                .expect("failed to deserialize");
            assert_eq!(FrameHeader::try_from(value), Err(FrameTypeError));
        }
    }
}
