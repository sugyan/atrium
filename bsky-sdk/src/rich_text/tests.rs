mod detection;

use crate::error::Result;
use crate::rich_text::{RichText, RichTextSegment};
use crate::tests::MockClient;
use atrium_api::app::bsky::richtext::facet::{ByteSlice, Link, Main, MainFeaturesItem, Mention};
use atrium_api::types::{Union, UnknownData, EMPTY_EXTRA_DATA};
use ipld_core::ipld::Ipld;

pub async fn rich_text_with_detect_facets(text: &str) -> Result<RichText> {
    #[cfg(feature = "default-client")]
    {
        let mut rt = RichText::new(text, None);
        rt.detect_facets(MockClient).await?;
        Ok(rt)
    }
    #[cfg(not(feature = "default-client"))]
    {
        RichText::new_with_detect_facets(text, MockClient).await
    }
}

fn facet(byte_start: usize, byte_end: usize) -> Main {
    Main {
        features: vec![Union::Unknown(UnknownData {
            r#type: String::new(),
            data: Ipld::Null,
        })],
        index: ByteSlice {
            byte_end,
            byte_start,
            extra_data: EMPTY_EXTRA_DATA,
        },
        extra_data: EMPTY_EXTRA_DATA,
    }
}

#[test]
fn calculate_bytelength_and_grapheme_length() {
    {
        let rt = RichText::new("Hello!", None);
        assert_eq!(rt.text.len(), 6);
        assert_eq!(rt.grapheme_len(), 6);
    }
    {
        let rt = RichText::new("👨‍👩‍👧‍👧", None);
        assert_eq!(rt.text.len(), 25);
        assert_eq!(rt.grapheme_len(), 1);
    }
    {
        let rt = RichText::new("👨‍👩‍👧‍👧🔥 good!✅", None);
        assert_eq!(rt.text.len(), 38);
        assert_eq!(rt.grapheme_len(), 9);
    }
}

#[test]
fn insert() {
    let input = &RichText::new("hello world", Some(vec![facet(2, 7)]));
    // correctly adjusts facets (scenario A - before)
    {
        let mut input = input.clone();
        input.insert(0, "test");
        assert_eq!(input.text, "testhello world");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 6);
        assert_eq!(facets[0].index.byte_end, 11);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "llo w"
        );
    }
    // correctly adjusts facets (scenario B - inner)
    {
        let mut input = input.clone();
        input.insert(4, "test");
        assert_eq!(input.text, "helltesto world");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 2);
        assert_eq!(facets[0].index.byte_end, 11);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "lltesto w"
        );
    }
    // correctly adjusts facets (scenario C - after)
    {
        let mut input = input.clone();
        input.insert(8, "test");
        assert_eq!(input.text, "hello wotestrld");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 2);
        assert_eq!(facets[0].index.byte_end, 7);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "llo w"
        );
    }
}

#[test]
fn insert_with_fat_unicode() {
    let input = &RichText::new(
        "one👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧",
        Some(vec![facet(0, 28), facet(29, 57), facet(58, 88)]),
    );
    // correctly adjusts facets (scenario A - before)
    {
        let mut input = input.clone();
        input.insert(0, "test");
        assert_eq!(input.text, "testone👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 3);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "one👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[1].index.byte_start..facets[1].index.byte_end],
            "two👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[2].index.byte_start..facets[2].index.byte_end],
            "three👨‍👩‍👧‍👧"
        );
    }
    // correctly adjusts facets (scenario B - inner)
    {
        let mut input = input.clone();
        input.insert(3, "test");
        assert_eq!(input.text, "onetest👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 3);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "onetest👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[1].index.byte_start..facets[1].index.byte_end],
            "two👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[2].index.byte_start..facets[2].index.byte_end],
            "three👨‍👩‍👧‍👧"
        );
    }
    // correctly adjusts facets (scenario C - after)
    {
        let mut input = input.clone();
        input.insert(28, "test");
        assert_eq!(input.text, "one👨‍👩‍👧‍👧test two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 3);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "one👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[1].index.byte_start..facets[1].index.byte_end],
            "two👨‍👩‍👧‍👧"
        );
        assert_eq!(
            &input.text[facets[2].index.byte_start..facets[2].index.byte_end],
            "three👨‍👩‍👧‍👧"
        );
    }
}

#[test]
fn delete() {
    let input = &RichText::new("hello world", Some(vec![facet(2, 7)]));
    // correctly adjusts facets (scenario A - entirely outer)
    {
        let mut input = input.clone();
        input.delete(0, 9);
        assert_eq!(input.text, "ld");
        let facets = input.facets.expect("facets should exist");
        assert!(facets.is_empty());
    }
    // correctly adjusts facets (scenario B - entirely after)
    {
        let mut input = input.clone();
        input.delete(7, 11);
        assert_eq!(input.text, "hello w");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 2);
        assert_eq!(facets[0].index.byte_end, 7);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "llo w"
        );
    }
    // correctly adjusts facets (scenario C - partially after)
    {
        let mut input = input.clone();
        input.delete(4, 11);
        assert_eq!(input.text, "hell");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 2);
        assert_eq!(facets[0].index.byte_end, 4);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "ll"
        );
    }
    // correctly adjusts facets (scenario D - entirely inner)
    {
        let mut input = input.clone();
        input.delete(3, 5);
        assert_eq!(input.text, "hel world");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 2);
        assert_eq!(facets[0].index.byte_end, 5);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "l w"
        );
    }
    // correctly adjusts facets (scenario E - partially before)
    {
        let mut input = input.clone();
        input.delete(1, 5);
        assert_eq!(input.text, "h world");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 1);
        assert_eq!(facets[0].index.byte_end, 3);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            " w"
        );
    }
    // correctly adjusts facets (scenario F - entirely before)
    {
        let mut input = input.clone();
        input.delete(0, 2);
        assert_eq!(input.text, "llo world");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 0);
        assert_eq!(facets[0].index.byte_end, 5);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "llo w"
        );
    }
}

#[test]
fn delete_with_fat_unicode() {
    let input = &RichText::new(
        "one👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧",
        Some(vec![facet(29, 57)]),
    );
    // correctly adjusts facets (scenario A - entirely outer)
    {
        let mut input = input.clone();
        input.delete(28, 58);
        assert_eq!(input.text, "one👨‍👩‍👧‍👧three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert!(facets.is_empty());
    }
    // correctly adjusts facets (scenario B - entirely after)
    {
        let mut input = input.clone();
        input.delete(57, 88);
        assert_eq!(input.text, "one👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 29);
        assert_eq!(facets[0].index.byte_end, 57);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "two👨‍👩‍👧‍👧"
        );
    }
    // correctly adjusts facets (scenario C - partially after)
    {
        let mut input = input.clone();
        input.delete(31, 88);
        assert_eq!(input.text, "one👨‍👩‍👧‍👧 tw");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 29);
        assert_eq!(facets[0].index.byte_end, 31);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "tw"
        );
    }
    // correctly adjusts facets (scenario D - entirely inner)
    {
        let mut input = input.clone();
        input.delete(30, 32);
        assert_eq!(input.text, "one👨‍👩‍👧‍👧 t👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 29);
        assert_eq!(facets[0].index.byte_end, 55);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "t👨‍👩‍👧‍👧"
        );
    }
    // correctly adjusts facets (scenario E - partially before)
    {
        let mut input = input.clone();
        input.delete(28, 31);
        assert_eq!(input.text, "one👨‍👩‍👧‍👧o👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 28);
        assert_eq!(facets[0].index.byte_end, 54);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "o👨‍👩‍👧‍👧"
        );
    }
    // correctly adjusts facets (scenario F - entirely before)
    {
        let mut input = input.clone();
        input.delete(0, 2);
        assert_eq!(input.text, "e👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧");
        let facets = input.facets.expect("facets should exist");
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 27);
        assert_eq!(facets[0].index.byte_end, 55);
        assert_eq!(
            &input.text[facets[0].index.byte_start..facets[0].index.byte_end],
            "two👨‍👩‍👧‍👧"
        );
    }
}

#[test]
fn segments() {
    // produces an empty output for an empty input
    {
        let input = RichText::new("", None);
        assert_eq!(input.segments(), vec![RichTextSegment::new("", None)]);
    }
    // produces a single segment when no facets are present
    {
        let input = RichText::new("hello", None);
        assert_eq!(input.segments(), vec![RichTextSegment::new("hello", None)]);
    }
    // produces 3 segments with 1 entity in the middle
    {
        let input = RichText::new("one two three", Some(vec![facet(4, 7)]));
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one ", None),
                RichTextSegment::new("two", Some(facet(4, 7))),
                RichTextSegment::new(" three", None),
            ]
        );
    }
    // produces 2 segments with 1 entity in the byteStart
    {
        let input = RichText::new("one two three", Some(vec![facet(0, 7)]));
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one two", Some(facet(0, 7))),
                RichTextSegment::new(" three", None),
            ]
        );
    }
    // produces 2 segments with 1 entity in the end
    {
        let input = RichText::new("one two three", Some(vec![facet(4, 13)]));
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one ", None),
                RichTextSegment::new("two three", Some(facet(4, 13))),
            ]
        );
    }
    // produces 1 segments with 1 entity around the entire string
    {
        let input = RichText::new("one two three", Some(vec![facet(0, 13)]));
        assert_eq!(
            input.segments(),
            vec![RichTextSegment::new("one two three", Some(facet(0, 13)))]
        );
    }
    // produces 5 segments with 3 facets covering each word
    {
        let input = RichText::new(
            "one two three",
            Some(vec![facet(0, 3), facet(4, 7), facet(8, 13)]),
        );
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one", Some(facet(0, 3))),
                RichTextSegment::new(" ", None),
                RichTextSegment::new("two", Some(facet(4, 7))),
                RichTextSegment::new(" ", None),
                RichTextSegment::new("three", Some(facet(8, 13))),
            ]
        );
    }
    // uses utf8 indices
    {
        let input = RichText::new(
            "one👨‍👩‍👧‍👧 two👨‍👩‍👧‍👧 three👨‍👩‍👧‍👧",
            Some(vec![facet(0, 28), facet(29, 57), facet(58, 88)]),
        );
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one👨‍👩‍👧‍👧", Some(facet(0, 28))),
                RichTextSegment::new(" ", None),
                RichTextSegment::new("two👨‍👩‍👧‍👧", Some(facet(29, 57))),
                RichTextSegment::new(" ", None),
                RichTextSegment::new("three👨‍👩‍👧‍👧", Some(facet(58, 88))),
            ]
        );
    }
    // correctly identifies mentions and links
    {
        let input = RichText::new(
            "one two three",
            Some(vec![
                Main {
                    features: vec![Union::Refs(MainFeaturesItem::Mention(Box::new(Mention {
                        did: "did:plc:123".parse().expect("invalid did"),
                        extra_data: EMPTY_EXTRA_DATA,
                    })))],
                    index: ByteSlice {
                        byte_end: 3,
                        byte_start: 0,
                        extra_data: EMPTY_EXTRA_DATA,
                    },
                    extra_data: EMPTY_EXTRA_DATA,
                },
                Main {
                    features: vec![Union::Refs(MainFeaturesItem::Link(Box::new(Link {
                        uri: String::from("https://example.com"),
                        extra_data: EMPTY_EXTRA_DATA,
                    })))],
                    index: ByteSlice {
                        byte_end: 7,
                        byte_start: 4,
                        extra_data: EMPTY_EXTRA_DATA,
                    },
                    extra_data: EMPTY_EXTRA_DATA,
                },
                facet(8, 13),
            ]),
        );
        let segments = input.segments();
        assert_eq!(segments.len(), 5);
        assert!(segments[0].link().is_none());
        assert!(segments[0].mention().is_some());
        assert!(segments[1].link().is_none());
        assert!(segments[1].mention().is_none());
        assert!(segments[2].link().is_some());
        assert!(segments[2].mention().is_none());
        assert!(segments[3].link().is_none());
        assert!(segments[3].mention().is_none());
        assert!(segments[4].link().is_none());
        assert!(segments[4].mention().is_none());
    }
    // skips facets that incorrectly overlap (left edge)
    {
        let input = RichText::new(
            "one two three",
            Some(vec![facet(0, 3), facet(2, 9), facet(8, 13)]),
        );
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one", Some(facet(0, 3))),
                RichTextSegment::new(" two ", None),
                RichTextSegment::new("three", Some(facet(8, 13))),
            ]
        );
    }
    // skips facets that incorrectly overlap (right edge)
    {
        let input = RichText::new(
            "one two three",
            Some(vec![facet(0, 3), facet(4, 9), facet(8, 13)]),
        );
        assert_eq!(
            input.segments(),
            vec![
                RichTextSegment::new("one", Some(facet(0, 3))),
                RichTextSegment::new(" ", None),
                RichTextSegment::new("two t", Some(facet(4, 9))),
                RichTextSegment::new("hree", None),
            ]
        );
    }
}
