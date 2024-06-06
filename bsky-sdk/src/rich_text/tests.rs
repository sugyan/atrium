mod detection;

use crate::rich_text::RichText;
use atrium_api::app::bsky::richtext::facet::{ByteSlice, Main};
use atrium_api::types::{Union, UnknownData};
use ipld_core::ipld::Ipld;

fn facet(byte_start: usize, byte_end: usize) -> Main {
    Main {
        features: vec![Union::Unknown(UnknownData {
            r#type: String::new(),
            data: Ipld::Null,
        })],
        index: ByteSlice {
            byte_start,
            byte_end,
        },
    }
}

#[test]
fn calculate_bytelength_and_grapheme_length() {
    {
        let rt = RichText::new("Hello!", None);
        assert_eq!(rt.len(), 6);
        assert_eq!(rt.grapheme_len(), 6);
    }
    {
        let rt = RichText::new("👨‍👩‍👧‍👧", None);
        assert_eq!(rt.len(), 25);
        assert_eq!(rt.grapheme_len(), 1);
    }
    {
        let rt = RichText::new("👨‍👩‍👧‍👧🔥 good!✅", None);
        assert_eq!(rt.len(), 38);
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
