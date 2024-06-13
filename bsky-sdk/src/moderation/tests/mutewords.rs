use super::{post_view, profile_view_basic};
use crate::moderation::decision::DecisionContext;
use crate::moderation::{ModerationPrefs, Moderator};
use atrium_api::app::bsky::actor::defs::MutedWord;
use std::collections::HashMap;

#[cfg(feature = "rich-text")]
#[tokio::test]
async fn has_muted_word_from_rich_text() -> crate::error::Result<()> {
    use crate::moderation::mutewords::has_muted_word;
    use crate::rich_text::tests::rich_text_with_detect_facets;

    // match: outline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("tag")],
                value: String::from("outlineTag"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![String::from("outlineTag")]),
            &None,
        ));
    }
    // match: inline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("tag")],
                value: String::from("inlineTag"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![String::from("outlineTag")]),
            &None,
        ));
    }
    // match: content target matches inline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("inlineTag"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![String::from("outlineTag")]),
            &None,
        ));
    }
    // no match: only tag targets
    {
        let rt = rich_text_with_detect_facets("This is a post").await?;
        assert!(!has_muted_word(
            &[MutedWord {
                targets: vec![String::from("tag")],
                value: String::from("post"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None,
        ));
    }
    // match: single character 希
    {
        let rt = rich_text_with_detect_facets("改善希望です").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("希"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None,
        ));
    }
    // match: single char with length > 1 ☠︎
    {
        let rt = rich_text_with_detect_facets("Idk why ☠︎ but maybe").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("☠︎"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // no match: long muted word, short post
    {
        let rt = rich_text_with_detect_facets("hey").await?;
        assert!(!has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("politics"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // match: exact text
    {
        let rt = rich_text_with_detect_facets("javascript").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("javascript"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // match: word within post
    {
        let rt = rich_text_with_detect_facets("This is a post about javascript").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("javascript"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // no match: partial word
    {
        let rt = rich_text_with_detect_facets("Use your brain, Eric").await?;
        assert!(!has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("ai"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // match: multiline
    {
        let rt = rich_text_with_detect_facets("Use your\n\tbrain, Eric").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("brain"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // match: :)
    {
        let rt = rich_text_with_detect_facets("So happy :)").await?;
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from(":)"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // yay!
    {
        let rt = rich_text_with_detect_facets("We're federating, yay!").await?;
        // match: yay!
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("yay!"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: yay
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("yay"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // y!ppee!!
    {
        let rt = rich_text_with_detect_facets("We're federating, y!ppee!!").await?;
        // match: y!ppee
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("y!ppee"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: y!ppee!
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("y!ppee!"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // apostrophes: Bluesky's
    {
        let rt = rich_text_with_detect_facets("Yay, Bluesky's mutewords work").await?;
        // match: Bluesky's
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("Bluesky's"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: Bluesky
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("Bluesky"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: bluesky
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("bluesky"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: blueskys
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("blueskys"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // Why so S@assy?
    {
        let rt = rich_text_with_detect_facets("Why so S@assy?").await?;
        // match: S@assy
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("S@assy"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: s@assy
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("s@assy"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // New York Times
    {
        let rt = rich_text_with_detect_facets("New York Times").await?;
        // match: new york times
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("new york times"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // !command
    {
        let rt = rich_text_with_detect_facets("Idk maybe a bot !command").await?;
        // match: !command
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("!command"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: command
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("command"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // no match: !command
        let rt = rich_text_with_detect_facets("Idk maybe a bot command").await?;
        assert!(!has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("!command"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // e/acc
    {
        let rt = rich_text_with_detect_facets("I'm e/acc pilled").await?;
        // match: e/acc
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("e/acc"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: acc
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("acc"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // super-bad
    {
        let rt = rich_text_with_detect_facets("I'm super-bad").await?;
        // match: super-bad
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("super-bad"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: super
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("super"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: bad
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("bad"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: super bad
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("super bad"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: superbad
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("superbad"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // idk_what_this_would_be
    {
        let rt = rich_text_with_detect_facets("Weird post with idk_what_this_would_be").await?;
        // match: idk what this would be
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("idk what this would be"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // no match: idk what this would be for
        assert!(!has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("idk what this would be for"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: idk
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("idk"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: idkwhatthiswouldbe
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("idkwhatthiswouldbe"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // parentheses
    {
        let rt = rich_text_with_detect_facets("Post with context(iykyk)").await?;
        // match: context(iykyk)
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("context(iykyk)"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: context
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("context"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: iykyk
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("iykyk"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: (iykyk)
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("(iykyk)"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // 🦋
    {
        let rt = rich_text_with_detect_facets("Post with 🦋").await?;
        // match: 🦋
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("🦋"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // phrases
    {
        let rt = rich_text_with_detect_facets(
            "I like turtles, or how I learned to stop worrying and love the internet.",
        )
        .await?;
        // match: stop worrying
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("stop worrying"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
        // match: turtles, or how
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("turtles, or how"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &None
        ));
    }
    // languages without spaces
    {
        let rt = rich_text_with_detect_facets("私はカメが好きです、またはどのようにして心配するのをやめてインターネットを愛するようになったのか").await?;
        // match: インターネット
        assert!(has_muted_word(
            &[MutedWord {
                targets: vec![String::from("content")],
                value: String::from("インターネット"),
            }],
            &rt.text,
            &rt.facets,
            &Some(vec![]),
            &Some(vec!["ja".parse().expect("invalid lang")],)
        ));
    }
    Ok(())
}

#[test]
fn does_not_mute_own_post() {
    let prefs = &ModerationPrefs {
        adult_content_enabled: false,
        labels: HashMap::new(),
        labelers: Vec::new(),
        muted_words: vec![MutedWord {
            targets: vec![String::from("content")],
            value: String::from("words"),
        }],
        hidden_posts: Vec::new(),
    };
    let post = &post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Mute words!",
        None,
    );
    // does mute if it isn't own post
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        prefs.clone(),
        HashMap::new(),
    );
    let result = moderator.moderate_post(post);
    assert!(
        result.ui(DecisionContext::ContentList).filter(),
        "post should be filtered"
    );
    // doesn't mute own post when muted word is in text
    let moderator = Moderator::new(
        Some("did:web:bob.test".parse().expect("invalid did")),
        prefs.clone(),
        HashMap::new(),
    );
    let result = moderator.moderate_post(post);
    assert!(
        !result.ui(DecisionContext::ContentList).filter(),
        "post should not be filtered"
    );
}

#[cfg(feature = "rich-text")]
#[tokio::test]
async fn does_not_mute_own_tags() -> crate::error::Result<()> {
    use crate::rich_text::tests::rich_text_with_detect_facets;
    use atrium_api::records::{KnownRecord, Record};

    let prefs = ModerationPrefs {
        adult_content_enabled: false,
        labels: HashMap::new(),
        labelers: Vec::new(),
        muted_words: vec![MutedWord {
            targets: vec![String::from("tag")],
            value: String::from("words"),
        }],
        hidden_posts: Vec::new(),
    };
    let rt = rich_text_with_detect_facets("Mute #words!").await?;
    let mut post = post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        &rt.text,
        None,
    );
    if let Record::Known(KnownRecord::AppBskyFeedPost(ref mut post)) = post.record {
        post.facets = rt.facets;
    }
    let moderator = Moderator::new(
        Some("did:web:bob.test".parse().expect("invalid did")),
        prefs,
        HashMap::new(),
    );
    let result = moderator.moderate_post(&post);
    assert!(
        !result.ui(DecisionContext::ContentList).filter(),
        "post should not be filtered"
    );
    Ok(())
}
