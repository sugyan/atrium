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
    let result = FrameHeader::try_from(ipld);
    assert_eq!(
        result.expect("failed to deserialize"),
        FrameHeader::Message {
            t: String::from("#commit")
        }
    );
}

#[test]
fn deserialize_error_frame_header() {
    // {"op": -1}
    let data = serialized_data("a1626f7020");
    let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
    let result = FrameHeader::try_from(ipld);
    assert_eq!(result.expect("failed to deserialize"), FrameHeader::Error);
}

#[test]
fn deserialize_invalid_frame_header() {
    {
        // {"op": 2, "t": "#commit"}
        let data = serialized_data("a2626f700261746723636f6d6d6974");
        let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
        let result = FrameHeader::try_from(ipld);
        assert_eq!(
            result.expect_err("must be failed").to_string(),
            "Unknown frame type. Header: {\"op\": 2, \"t\": \"#commit\"}"
        );
    }
    {
        // {"op": -2}
        let data = serialized_data("a1626f7021");
        let ipld = serde_ipld_dagcbor::from_slice::<Ipld>(&data).expect("failed to deserialize");
        let result = FrameHeader::try_from(ipld);
        assert_eq!(
            result.expect_err("must be failed").to_string(),
            "Unknown frame type. Header: {\"op\": -2}"
        );
    }
}
