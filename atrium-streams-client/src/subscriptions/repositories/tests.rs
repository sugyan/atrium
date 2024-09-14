use std::{convert::identity as id, vec};

use atrium_streams::{
    atrium_api::{
        com::atproto::{
            label::subscribe_labels::InfoData,
            sync::subscribe_repos::{
                self, AccountData, CommitData, HandleData, IdentityData, MigrateData, TombstoneData,
            },
        },
        types::{
            string::{Datetime, Did, Handle},
            CidLink, Object,
        },
    },
    subscriptions::{
        handlers::repositories::HandledData, ConnectionHandler, ProcessedPayload, SubscriptionError,
    },
};
use futures::{executor::block_on_stream, Stream};
use ipld_core::{
    cid::{multihash::Multihash, Cid},
    ipld::Ipld,
};
use serde_json::Value;
use tokio_tungstenite::tungstenite::{Error, Message};

use super::{firehose::Firehose, Repositories};

fn serialize_ipld(frame: &str) -> Result<Vec<u8>, anyhow::Error> {
    if frame.is_empty() {
        return Ok(vec![]);
    }

    let json: Value = serde_json::from_str(frame)?;
    let bytes = serde_ipld_dagcbor::to_vec(&json)?;
    Ok(bytes)
}

fn mock_connection<'a>(
    packets: Vec<(&'a str, &'a str)>,
) -> impl Stream<Item = Result<Message, Error>> + Unpin + 'a {
    let mut stream = packets.into_iter().map(|(header, payload)| {
        // Using Utf8 as an arbitrary tungstenite error
        serialize_ipld(header)
            .map(|mut v| {
                serialize_ipld(payload)
                    .map(|mut p| {
                        Message::Binary({
                            v.append(&mut p);
                            v
                        })
                    })
                    .map_err(|_| Error::Utf8)
            })
            .map_err(|_| Error::Utf8)
            .and_then(id)
    });
    let connection = async_stream::stream! {
        while let Some(packet) = stream.next() {
            yield packet;
        }
    };

    Box::pin(connection)
}

fn test_packet(
    packet: Option<(&str, &str)>,
) -> Option<Result<(Option<i64>, HandledData<Firehose>), SubscriptionError<subscribe_repos::Error>>>
{
    let connection = mock_connection(if let Some(packet) = packet {
        vec![packet]
    } else {
        vec![]
    });

    let subscription = gen_default_subscription(connection);

    block_on_stream(subscription)
        .next()
        .map(|v| v.map(|ProcessedPayload { data, seq }| (seq, data)))
}

fn gen_default_subscription(
    connection: impl Stream<Item = Result<Message, Error>> + Unpin,
) -> impl Stream<
    Item = Result<
        ProcessedPayload<<Firehose as ConnectionHandler>::HandledData>,
        SubscriptionError<subscribe_repos::Error>,
    >,
> {
    let firehose = Firehose::builder()
        .enable_commit(true)
        .enable_identity(true)
        .enable_account(true)
        .enable_handle(true)
        .enable_migrate(true)
        .enable_tombstone(true)
        .enable_info(true)
        .build();
    let subscription = Repositories::builder()
        .connection(connection)
        .handler(firehose)
        .build();
    subscription
}

#[test]
fn disconnect() {
    if test_packet(None).is_none() {
        return;
    }
    panic!("Expected None")
}

#[test]
fn invalid_packet() {
    if let SubscriptionError::Abort(msg) =
        test_packet(Some(("{ not-a-header }", "{ not-a-payload }")))
            .unwrap()
            .unwrap_err()
    {
        assert_eq!(msg, "Received invalid packet. Error: Utf8");
        return;
    }
    panic!("Expected Invalid Packet")
}

#[test]
fn commit() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: Some(CommitData {
            blobs: vec![],
            blocks: vec![],
            commit: CidLink(Cid::new_v1(
                0x70,
                Multihash::<64>::wrap(0x12, &[0; 64]).unwrap(),
            )),
            ops: vec![],
            prev: None,
            rebase: false,
            repo: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            rev: String::new(),
            seq: 99,
            since: None,
            time: now,
            too_big: true,
        }),
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#commit" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Commit(ProcessedCommitData {{ \
                repo: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                commit: CidLink(Cid(bafybeqaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa)), \
                ops: None, \
                blobs: [], \
                rev: \"\", \
                since: None, \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn identity() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: IdentityData {
            did: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            handle: None,
            seq: 99,
            time: now,
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#identity" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Identity(ProcessedIdentityData {{ \
                did: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                handle: None, \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn account() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: AccountData {
            active: false,
            did: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            seq: 99,
            status: None,
            time: now,
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#account" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Account(ProcessedAccountData {{ \
                did: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                active: false, \
                status: None, \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn handle() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: HandleData {
            did: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            handle: Handle::new("test.handle.xyz".to_string()).unwrap(),
            seq: 99,
            time: now,
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#handle" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Handle(ProcessedHandleData {{ \
                did: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                handle: Handle(\"test.handle.xyz\"), \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn migrate() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: MigrateData {
            did: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            migrate_to: None,
            seq: 99,
            time: now,
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#migrate" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Migrate(ProcessedMigrateData {{ \
                did: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                migrate_to: None, \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn tombstone() {
    let now = Datetime::now();
    let now_str = format!("{:?}", now);
    let body = Object {
        data: TombstoneData {
            did: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            seq: 99,
            time: now,
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#tombstone" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, Some(99));
    assert_eq!(
        format!("{:?}", data),
        format!(
            "Tombstone(ProcessedTombstoneData {{ \
                did: Did(\"did:plc:ewvi7nxzyoun6zhxrhs64oiz\"), \
                time: {now_str} \
            }})"
        )
    );
}

#[test]
fn info() {
    let body = Object {
        data: InfoData {
            message: Some("Requested cursor exceeded limit. Possibly missing events".to_string()),
            name: "OutdatedCursor".to_string(),
        },
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    let (seq, data) = test_packet(Some((r##"{ "op": 1, "t": "#info" }"##, &body)))
        .unwrap()
        .unwrap();
    assert_eq!(seq, None);
    assert_eq!(
        format!("{:?}", data),
        "Info(InfoData { \
            message: Some(\"Requested cursor exceeded limit. Possibly missing events\"), \
            name: \"OutdatedCursor\" \
        })"
        .to_string()
    );
}

#[test]
fn ignored_frame() {
    if test_packet(Some((
        r##"{ "op": 1, "t": "#non-existent" }"##,
        r#"{ "foo": "bar" }"#,
    )))
    .is_none()
    {
        return;
    }
    panic!("Expected None")
}

#[test]
fn invalid_body() {
    let body = Object {
        data: Some(CommitData {
            blobs: vec![],
            blocks: vec![1], // Invalid CAR file
            commit: CidLink(Cid::new_v1(
                0x70,
                Multihash::<64>::wrap(0x12, &[0; 64]).unwrap(),
            )),
            ops: vec![],
            prev: None,
            rebase: false,
            repo: Did::new("did:plc:ewvi7nxzyoun6zhxrhs64oiz".to_string()).unwrap(),
            rev: String::new(),
            seq: 0,
            since: None,
            time: Datetime::now(),
            too_big: false,
        }),
        extra_data: Ipld::Null,
    };
    let body = serde_json::to_string(&body).unwrap();
    if let SubscriptionError::Abort(msg) =
        test_packet(Some((r##"{ "op": 1, "t": "#commit" }"##, &body)))
            .unwrap()
            .unwrap_err()
    {
        assert_eq!(
            msg,
            "Received invalid payload. Error: CarDecoding(IoError(Kind(UnexpectedEof)))"
        );
        return;
    }
}

#[test]
fn future_cursor() {
    let res = test_packet(Some((
        r##"{ "op": -1 }"##,
        r#"{ "error": "FutureCursor", "message": "Cursor in the future." }"#,
    )));
    if let SubscriptionError::Other(subscribe_repos::Error::FutureCursor(Some(s))) =
        res.unwrap().unwrap_err()
    {
        assert_eq!("Cursor in the future.", &s);
        return;
    }
    panic!("Expected FutureCursor")
}

#[test]
fn consumer_too_slow() {
    let res = test_packet(Some((
        r##"{ "op": -1 }"##,
        r#"{ "error": "ConsumerTooSlow", "message": "Stream consumer too slow" }"#,
    )));
    if let SubscriptionError::Other(subscribe_repos::Error::ConsumerTooSlow(Some(s))) =
        res.unwrap().unwrap_err()
    {
        assert_eq!("Stream consumer too slow", &s);
        return;
    }
    panic!("Expected ConsumerTooSlow")
}

#[test]
fn unknown_error() {
    let res = test_packet(Some((
        r##"{ "op": -1 }"##,
        r#"{ "error": "Unknown", "message": "No one knows" }"#,
    )));
    if let SubscriptionError::Unknown(msg) = res.unwrap().unwrap_err() {
        assert_eq!(
            "Failed to decode error frame: \
                Msg(\"unknown variant `Unknown`, expected `FutureCursor` or `ConsumerTooSlow`\")",
            &msg
        );
        return;
    }
    panic!("Expected Unknown")
}

#[test]
fn empty_payload() {
    let res = test_packet(Some((r##"{ "op": 1, "t": "#commit" }"##, r#""#)));
    if let SubscriptionError::Abort(msg) = res.unwrap().unwrap_err() {
        assert_eq!(
            "Received empty payload for header: {\"op\": 1, \"t\": \"#commit\"}",
            &msg
        );
        return;
    }
    panic!("Expected Empty Payload")
}

#[test]
fn invalid_frame() {
    fn mock_invalid() -> impl Stream<Item = Result<Message, Error>> + Unpin {
        let mut stream = vec![Message::Binary(vec![b'{'])].into_iter();
        let connection = async_stream::stream! {
            while let Some(packet) = stream.next() {
                yield Ok(packet);
            }
        };
        Box::pin(connection)
    }

    let subscription = gen_default_subscription(mock_invalid());

    let res = block_on_stream(subscription)
        .next()
        .map(|v| v.map(|ProcessedPayload { data, seq }| (seq, data)));

    if let SubscriptionError::Abort(msg) = res.unwrap().unwrap_err() {
        assert_eq!("Received invalid frame. Error: Eof", &msg);
        return;
    }
    panic!("Expected Invalid Frame")
}

#[test]
fn unknown_frame() {
    let res = test_packet(Some((r##"{ "op": 2 }"##, r#"{ "unknown": "header" }"#)));
    if res.is_none() {
        return;
    }
    panic!("Expected None")
}
