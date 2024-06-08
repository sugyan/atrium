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
    ];
    for (input, expected) in test_cases {
        let mut rt = RichText::new(input, None);
        rt.detect_facets(MockClient).await?;
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
