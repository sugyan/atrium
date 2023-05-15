use atrium_api::com::atproto::sync::subscribe_repos::Commit;
use ciborium::{from_reader, value::Integer, Value};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite};

enum FrameType {
    Message,
    Error,
}

#[derive(Debug, Clone)]
struct FrameTypeError;

impl TryFrom<Integer> for FrameType {
    type Error = FrameTypeError;

    fn try_from(value: Integer) -> Result<Self, <FrameType as TryFrom<Integer>>::Error> {
        match i8::try_from(value) {
            Ok(1) => Ok(Self::Message),
            Ok(-1) => Ok(Self::Error),
            _ => Err(FrameTypeError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MessageFrameHeader {
    t: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ErrorFrameHeader {}

#[derive(Debug, Clone)]
enum FrameHeader {
    Message(MessageFrameHeader),
    Error(ErrorFrameHeader),
}

#[derive(Debug, Clone)]
enum Frame {
    Message(MessageFrame),
    Error(ErrorFrame),
}

#[derive(Debug, Clone)]
struct MessageFrame {
    body: MessageEnum,
}

#[derive(Debug, Clone)]
struct ErrorFrame {
    // body: Value,
}

#[derive(Debug, Clone)]
enum MessageEnum {
    Commit(Commit),
    // Handle(Handle),
    // Migrate(Migrate),
    // Tombstone(Tombstone),
    // Info(Info),
}

impl Frame {
    fn cbor_decode_multi(data: &[u8]) -> Result<Vec<Value>, ciborium::de::Error<std::io::Error>> {
        let mut cursor = std::io::Cursor::new(data);
        let mut values = Vec::new();
        loop {
            match from_reader::<Value, _>(&mut cursor) {
                Ok(value) => values.push(value),
                Err(ciborium::de::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(values)
    }
    fn parse_header(value: &Value) -> Result<FrameHeader, Box<dyn std::error::Error>> {
        let op = value.as_map().and_then(|m| {
            m.iter().find_map(|(k, v)| {
                if k.as_text() == Some("op") {
                    Some(v)
                } else {
                    None
                }
            })
        });
        if let Some(Value::Integer(i)) = op {
            let mut buf = Vec::new();
            ciborium::ser::into_writer(value, &mut buf)?;
            match FrameType::try_from(*i) {
                Ok(FrameType::Message) => {
                    return Ok(FrameHeader::Message(from_reader(buf.as_slice())?));
                }
                Ok(FrameType::Error) => {
                    return Ok(FrameHeader::Error(from_reader(buf.as_slice())?));
                }
                _ => {}
            }
        }
        panic!("Invalid frame header.") // TODO
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, <Frame as TryFrom<&[u8]>>::Error> {
        let values = Self::cbor_decode_multi(value)?;
        if values.len() <= 1 {
            panic!("Missing frame body"); // TODO
        }
        if values.len() > 2 {
            panic!("Too many CBOR data items in frame"); // TODO
        }
        Ok(match Self::parse_header(&values[0])? {
            FrameHeader::Message(header) => {
                let mut buf = Vec::new();
                ciborium::ser::into_writer(&values[1], &mut buf)?;
                match header.t.as_deref() {
                    Some("#commit") => Self::Message(MessageFrame {
                        body: MessageEnum::Commit(from_reader(buf.as_slice())?),
                    }),
                    _ => unimplemented!(),
                }
            }
            FrameHeader::Error(_) => {
                // TODO
                Self::Error(ErrorFrame {})
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut stream, _) =
        connect_async("wss://bsky.social/xrpc/com.atproto.sync.subscribeRepos").await?;

    while let Some(Ok(tungstenite::Message::Binary(message))) = stream.next().await {
        match Frame::try_from(message.as_slice())? {
            Frame::Message(message) => {
                println!("{:?}", message.body);
            }
            Frame::Error(err) => panic!("{err:?}"),
        }
    }
    Ok(())
}
