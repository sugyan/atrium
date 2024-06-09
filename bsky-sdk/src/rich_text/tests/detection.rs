use crate::error::Result;
use crate::rich_text::{RichText, RichTextSegment};
use async_trait::async_trait;
use atrium_api::app::bsky::richtext::facet::MainFeaturesItem;
use atrium_api::types::Union;
use atrium_api::xrpc::types::Header;
use atrium_api::xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};

struct MockClient;

#[async_trait]
impl HttpClient for MockClient {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        if let Some(handle) = request
            .uri()
            .query()
            .and_then(|s| s.strip_prefix("handle="))
        {
            Ok(Response::builder()
                .status(200)
                .header(Header::ContentType, "application/json")
                .body(
                    format!(r#"{{"did": "did:fake:{}"}}"#, handle)
                        .as_bytes()
                        .to_vec(),
                )?)
        } else {
            Ok(Response::builder().status(500).body(Vec::new())?)
        }
    }
}

#[async_trait]
impl XrpcClient for MockClient {
    fn base_uri(&self) -> String {
        String::new()
    }
}

fn segment_to_output(segment: &RichTextSegment) -> (&str, Option<&str>) {
    (
        &segment.text,
        segment.facet.as_ref().and_then(|facet| {
            facet.features.iter().find_map(|feature| match feature {
                Union::Refs(MainFeaturesItem::Mention(mention)) => Some(mention.did.as_ref()),
                Union::Refs(MainFeaturesItem::Link(link)) => Some(&link.uri),
                Union::Refs(MainFeaturesItem::Tag(tag)) => Some(&tag.tag),
                _ => None,
            })
        }),
    )
}

#[tokio::test]
async fn detect_facets() -> Result<()> {
    let test_cases = [
        ("no mention", vec![("no mention", None)]),
        (
            "@handle.com middle end",
            vec![
                ("@handle.com", Some("did:fake:handle.com")),
                (" middle end", None),
            ],
        ),
        (
            "start @handle.com end",
            vec![
                ("start ", None),
                ("@handle.com", Some("did:fake:handle.com")),
                (" end", None),
            ],
        ),
        (
            "start middle @handle.com",
            vec![
                ("start middle ", None),
                ("@handle.com", Some("did:fake:handle.com")),
            ],
        ),
        (
            "@handle.com @handle.com @handle.com",
            vec![
                ("@handle.com", Some("did:fake:handle.com")),
                (" ", None),
                ("@handle.com", Some("did:fake:handle.com")),
                (" ", None),
                ("@handle.com", Some("did:fake:handle.com")),
            ],
        ),
        (
            "@full123-chars.test",
            vec![("@full123-chars.test", Some("did:fake:full123-chars.test"))],
        ),
        ("not@right", vec![("not@right", None)]),
        (
            "@handle.com!@#$chars",
            vec![
                ("@handle.com", Some("did:fake:handle.com")),
                ("!@#$chars", None),
            ],
        ),
        (
            "@handle.com\n@handle.com",
            vec![
                ("@handle.com", Some("did:fake:handle.com")),
                ("\n", None),
                ("@handle.com", Some("did:fake:handle.com")),
            ],
        ),
        (
            "parenthetical (@handle.com)",
            vec![
                ("parenthetical (", None),
                ("@handle.com", Some("did:fake:handle.com")),
                (")", None),
            ],
        ),
        (
            "👨‍👩‍👧‍👧 @handle.com 👨‍👩‍👧‍👧",
            vec![
                ("👨‍👩‍👧‍👧 ", None),
                ("@handle.com", Some("did:fake:handle.com")),
                (" 👨‍👩‍👧‍👧", None),
            ],
        ),
        (
            "start https://middle.com end",
            vec![
                ("start ", None),
                ("https://middle.com", Some("https://middle.com")),
                (" end", None),
            ],
        ),
        (
            "start https://middle.com/foo/bar end",
            vec![
                ("start ", None),
                (
                    "https://middle.com/foo/bar",
                    Some("https://middle.com/foo/bar"),
                ),
                (" end", None),
            ],
        ),
        (
            "start https://middle.com/foo/bar?baz=bux end",
            vec![
                ("start ", None),
                (
                    "https://middle.com/foo/bar?baz=bux",
                    Some("https://middle.com/foo/bar?baz=bux"),
                ),
                (" end", None),
            ],
        ),
        (
            "start https://middle.com/foo/bar?baz=bux#hash end",
            vec![
                ("start ", None),
                (
                    "https://middle.com/foo/bar?baz=bux#hash",
                    Some("https://middle.com/foo/bar?baz=bux#hash"),
                ),
                (" end", None),
            ],
        ),
        (
            "https://start.com/foo/bar?baz=bux#hash middle end",
            vec![
                (
                    "https://start.com/foo/bar?baz=bux#hash",
                    Some("https://start.com/foo/bar?baz=bux#hash"),
                ),
                (" middle end", None),
            ],
        ),
        (
            "start middle https://end.com/foo/bar?baz=bux#hash",
            vec![
                ("start middle ", None),
                (
                    "https://end.com/foo/bar?baz=bux#hash",
                    Some("https://end.com/foo/bar?baz=bux#hash"),
                ),
            ],
        ),
        (
            "https://newline1.com\nhttps://newline2.com",
            vec![
                ("https://newline1.com", Some("https://newline1.com")),
                ("\n", None),
                ("https://newline2.com", Some("https://newline2.com")),
            ],
        ),
        (
            "👨‍👩‍👧‍👧 https://middle.com 👨‍👩‍👧‍👧",
            vec![
                ("👨‍👩‍👧‍👧 ", None),
                ("https://middle.com", Some("https://middle.com")),
                (" 👨‍👩‍👧‍👧", None),
            ],
        ),
        (
            "start middle.com end",
            vec![
                ("start ", None),
                ("middle.com", Some("https://middle.com")),
                (" end", None),
            ],
        ),
        (
            "start middle.com/foo/bar end",
            vec![
                ("start ", None),
                ("middle.com/foo/bar", Some("https://middle.com/foo/bar")),
                (" end", None),
            ],
        ),
        (
            "start middle.com/foo/bar?baz=bux end",
            vec![
                ("start ", None),
                (
                    "middle.com/foo/bar?baz=bux",
                    Some("https://middle.com/foo/bar?baz=bux"),
                ),
                (" end", None),
            ],
        ),
        (
            "start middle.com/foo/bar?baz=bux#hash end",
            vec![
                ("start ", None),
                (
                    "middle.com/foo/bar?baz=bux#hash",
                    Some("https://middle.com/foo/bar?baz=bux#hash"),
                ),
                (" end", None),
            ],
        ),
        (
            "start.com/foo/bar?baz=bux#hash middle end",
            vec![
                (
                    "start.com/foo/bar?baz=bux#hash",
                    Some("https://start.com/foo/bar?baz=bux#hash"),
                ),
                (" middle end", None),
            ],
        ),
        (
            "start middle end.com/foo/bar?baz=bux#hash",
            vec![
                ("start middle ", None),
                (
                    "end.com/foo/bar?baz=bux#hash",
                    Some("https://end.com/foo/bar?baz=bux#hash"),
                ),
            ],
        ),
        (
            "newline1.com\nnewline2.com",
            vec![
                ("newline1.com", Some("https://newline1.com")),
                ("\n", None),
                ("newline2.com", Some("https://newline2.com")),
            ],
        ),
        (
            "a example.com/index.php php link",
            vec![
                ("a ", None),
                (
                    "example.com/index.php",
                    Some("https://example.com/index.php"),
                ),
                (" php link", None),
            ],
        ),
        (
            "a trailing bsky.app: colon",
            vec![
                ("a trailing ", None),
                ("bsky.app", Some("https://bsky.app")),
                (": colon", None),
            ],
        ),
        ("not.. a..url ..here", vec![("not.. a..url ..here", None)]),
        ("e.g.", vec![("e.g.", None)]),
        ("something-cool.jpg", vec![("something-cool.jpg", None)]),
        ("website.com.jpg", vec![("website.com.jpg", None)]),
        ("e.g./foo", vec![("e.g./foo", None)]),
        ("website.com.jpg/foo", vec![("website.com.jpg/foo", None)]),
    ];
    for (input, expected) in test_cases {
        let rt = RichText::new(input, None).detect_facets(MockClient).await?;
        assert_eq!(
            rt.segments()
                .iter()
                .map(segment_to_output)
                .collect::<Vec<_>>(),
            expected
        );
    }
    Ok(())
}
