use super::rich_text_with_detect_facets;
use crate::error::Result;
use crate::rich_text::RichTextSegment;
use atrium_api::app::bsky::richtext::facet::MainFeaturesItem;
use atrium_api::types::Union;

#[tokio::test]
async fn detect_facets_mentions_and_links() -> Result<()> {
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
        (
            "Classic article https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/",
            vec![
                ("Classic article ", None),
                (
                    "https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/",
                    Some("https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/"),
                ),
            ],
        ),
        (
            "Classic article https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/ ",
            vec![
                ("Classic article ", None),
                (
                    "https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/",
                    Some("https://socket3.wordpress.com/2018/02/03/designing-windows-95s-user-interface/"),
                ),
                (" ", None),
            ],
        ),
        (
            "https://foo.com https://bar.com/whatever https://baz.com",
            vec![
                ("https://foo.com", Some("https://foo.com")),
                (" ", None),
                ("https://bar.com/whatever", Some("https://bar.com/whatever")),
                (" ", None),
                ("https://baz.com", Some("https://baz.com")),
            ],
        ),
        (
            "punctuation https://foo.com, https://bar.com/whatever; https://baz.com.",
            vec![
                ("punctuation ", None),
                ("https://foo.com", Some("https://foo.com")),
                (", ", None),
                ("https://bar.com/whatever", Some("https://bar.com/whatever")),
                ("; ", None),
                ("https://baz.com", Some("https://baz.com")),
                (".", None),
            ],
        ),
        (
            "parenthentical (https://foo.com)",
            vec![
                ("parenthentical (", None),
                ("https://foo.com", Some("https://foo.com")),
                (")", None),
            ],
        ),
        (
            "except for https://foo.com/thing_(cool)",
            vec![
                ("except for ", None),
                (
                    "https://foo.com/thing_(cool)",
                    Some("https://foo.com/thing_(cool)"),
                ),
            ],
        ),
    ];
    fn segment_to_output(segment: &RichTextSegment) -> (&str, Option<&str>) {
        (
            &segment.text,
            segment.facet.as_ref().and_then(|facet| {
                facet.features.iter().find_map(|feature| match feature {
                    Union::Refs(MainFeaturesItem::Mention(mention)) => Some(mention.did.as_ref()),
                    Union::Refs(MainFeaturesItem::Link(link)) => Some(&link.uri),
                    _ => None,
                })
            }),
        )
    }
    for (input, expected) in test_cases {
        let rt = rich_text_with_detect_facets(input).await?;
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

#[tokio::test]
async fn detect_facets_tags() -> Result<()> {
    let test_cases = [
        ("#a", vec![("a", (0, 2))]),
        ("#a #b", vec![("a", (0, 2)), ("b", (3, 5))]),
        ("#1", vec![]),
        ("#1a", vec![("1a", (0, 3))]),
        ("#tag", vec![("tag", (0, 4))]),
        ("body #tag", vec![("tag", (5, 9))]),
        ("#tag body", vec![("tag", (0, 4))]),
        ("body #tag body", vec![("tag", (5, 9))]),
        ("body #1", vec![]),
        ("body #1a", vec![("1a", (5, 8))]),
        ("body #a1", vec![("a1", (5, 8))]),
        ("#", vec![]),
        ("#?", vec![]),
        ("text #", vec![]),
        ("text # text", vec![]),
        (
            "body #thisisa64characterstring_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            vec![(
                "thisisa64characterstring_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                (5, 70),
            )],
        ),
        (
            "body #thisisa65characterstring_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab",
            vec![],
        ),
        (
            "body #thisisa64characterstring_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!",
            vec![(
                "thisisa64characterstring_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                (5, 70),
            )],
        ),
        ("its a #double#rainbow", vec![("double#rainbow", (6, 21))]),
        ("##hashash", vec![("#hashash", (0, 9))]),
        ("##", vec![]),
        ("some #n0n3s@n5e!", vec![("n0n3s@n5e", (5, 15))]),
        (
            "works #with,punctuation",
            vec![("with,punctuation", (6, 23))],
        ),
        (
            "strips trailing #punctuation, #like. #this!",
            vec![
                ("punctuation", (16, 28)),
                ("like", (30, 35)),
                ("this", (37, 42)),
            ],
        ),
        (
            "strips #multi_trailing___...",
            vec![("multi_trailing", (7, 22))],
        ),
        (
            "works with #🦋 emoji, and #butter🦋fly",
            vec![("🦋", (11, 16)), ("butter🦋fly", (28, 42))],
        ),
        (
            "#same #same #but #diff",
            vec![
                ("same", (0, 5)),
                ("same", (6, 11)),
                ("but", (12, 16)),
                ("diff", (17, 22)),
            ],
        ),
        ("this #️⃣tag should not be a tag", vec![]),
        ("this ##️⃣tag should be a tag", vec![("#️⃣tag", (5, 16))]),
        ("this #t\nag should be a tag", vec![("t", (5, 7))]),
        #[allow(clippy::invisible_characters)]
        ("no match (\\u200B): #​", vec![]),
        #[allow(clippy::invisible_characters)]
        ("no match (\\u200Ba): #​a", vec![]),
        #[allow(clippy::invisible_characters)]
        ("match (a\\u200Bb): #a​b", vec![("a", (18, 20))]),
        #[allow(clippy::invisible_characters)]
        ("match (ab\\u200B): #ab​", vec![("ab", (18, 21))]),
        ("no match (\\u20e2tag): #⃢tag", vec![]),
        ("no match (a\\u20e2b): #a⃢b", vec![("a", (21, 23))]),
        (
            "match full width number sign (tag): ＃tag",
            vec![("tag", (36, 42))],
        ),
        (
            "match full width number sign (tag): ＃#️⃣tag",
            vec![("#️⃣tag", (36, 49))],
        ),
        ("no match 1?: #1?", vec![]),
    ];
    fn segment_to_output(segment: &RichTextSegment) -> Option<(&str, (usize, usize))> {
        segment.facet.as_ref().and_then(|facet| {
            facet.features.iter().find_map(|feature| match feature {
                Union::Refs(MainFeaturesItem::Tag(tag)) => Some((
                    tag.tag.as_ref(),
                    (facet.index.byte_start, facet.index.byte_end),
                )),
                _ => None,
            })
        })
    }

    for (input, expected) in test_cases {
        let rt = rich_text_with_detect_facets(input).await?;
        assert_eq!(
            rt.segments()
                .iter()
                .filter_map(segment_to_output)
                .collect::<Vec<_>>(),
            expected
        );
    }
    Ok(())
}
