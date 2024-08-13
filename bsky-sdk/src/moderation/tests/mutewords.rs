use super::{post_view, profile_view_basic};
use crate::moderation::decision::DecisionContext;
use crate::moderation::mutewords::has_muted_word;
use crate::moderation::{ModerationPrefs, Moderator};
use atrium_api::app::bsky::actor::defs::{MutedWord, MutedWordData, ViewerState, ViewerStateData};
use atrium_api::app::bsky::richtext::facet::{ByteSliceData, MainData, MainFeaturesItem, TagData};
use atrium_api::types::string::Datetime;
use atrium_api::types::{Union, UnknownData};
use ipld_core::ipld::Ipld;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

enum MutedWordTarget {
    Content,
    #[cfg(feature = "rich-text")]
    Tag,
}

enum ActorTarget {
    All,
    ExcludeFollowing,
}

fn muted_word(value: &str, word_target: MutedWordTarget, actor_target: ActorTarget) -> MutedWord {
    MutedWordData {
        actor_target: Some(match actor_target {
            ActorTarget::All => String::from("all"),
            ActorTarget::ExcludeFollowing => String::from("exclude-following"),
        }),
        expires_at: None,
        id: None,
        targets: vec![match word_target {
            MutedWordTarget::Content => String::from("content"),
            #[cfg(feature = "rich-text")]
            MutedWordTarget::Tag => String::from("tag"),
        }],
        value: String::from(value),
    }
    .into()
}

fn moderation_prefs(
    value: &str,
    word_target: MutedWordTarget,
    actor_target: ActorTarget,
    expires_at: Option<Datetime>,
) -> ModerationPrefs {
    let mut muted_word = muted_word(value, word_target, actor_target);
    muted_word.expires_at = expires_at;
    ModerationPrefs {
        adult_content_enabled: false,
        labels: HashMap::new(),
        labelers: Vec::new(),
        muted_words: vec![muted_word],
        hidden_posts: Vec::new(),
    }
}

fn viewer_state(following: Option<String>) -> ViewerState {
    ViewerStateData {
        blocked_by: None,
        blocking: None,
        blocking_by_list: None,
        followed_by: None,
        following,
        known_followers: None,
        muted: None,
        muted_by_list: None,
    }
    .into()
}

#[cfg(feature = "rich-text")]
#[tokio::test]
async fn has_muted_word_from_rich_text() -> crate::error::Result<()> {
    use crate::rich_text::tests::rich_text_with_detect_facets;
    // match: outline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[muted_word(
                "outlineTag",
                MutedWordTarget::Tag,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![String::from("outlineTag")]),
            None,
            None
        ));
    }
    // match: inline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[muted_word(
                "inlineTag",
                MutedWordTarget::Tag,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![String::from("outlineTag")]),
            None,
            None
        ));
    }
    // match: content target matches inline tag
    {
        let rt = rich_text_with_detect_facets("This is a post #inlineTag").await?;
        assert!(has_muted_word(
            &[muted_word(
                "inlineTag",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![String::from("outlineTag")]),
            None,
            None
        ));
    }
    // no match: only tag targets
    {
        let rt = rich_text_with_detect_facets("This is a post").await?;
        assert!(!has_muted_word(
            &[muted_word("post", MutedWordTarget::Tag, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: single character 希
    {
        let rt = rich_text_with_detect_facets("改善希望です").await?;
        assert!(has_muted_word(
            &[muted_word("希", MutedWordTarget::Content, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: single char with length > 1 ☠︎
    {
        let rt = rich_text_with_detect_facets("Idk why ☠︎ but maybe").await?;
        assert!(has_muted_word(
            &[muted_word("☠︎", MutedWordTarget::Content, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // no match: long muted word, short post
    {
        let rt = rich_text_with_detect_facets("hey").await?;
        assert!(!has_muted_word(
            &[muted_word(
                "politics",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: exact text
    {
        let rt = rich_text_with_detect_facets("javascript").await?;
        assert!(has_muted_word(
            &[muted_word(
                "javascript",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: word within post
    {
        let rt = rich_text_with_detect_facets("This is a post about javascript").await?;
        assert!(has_muted_word(
            &[muted_word(
                "javascript",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // no match: partial word
    {
        let rt = rich_text_with_detect_facets("Use your brain, Eric").await?;
        assert!(!has_muted_word(
            &[muted_word("ai", MutedWordTarget::Content, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: multiline
    {
        let rt = rich_text_with_detect_facets("Use your\n\tbrain, Eric").await?;
        assert!(has_muted_word(
            &[muted_word(
                "brain",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // match: :)
    {
        let rt = rich_text_with_detect_facets("So happy :)").await?;
        assert!(has_muted_word(
            &[muted_word(":)", MutedWordTarget::Content, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // yay!
    {
        let rt = rich_text_with_detect_facets("We're federating, yay!").await?;
        // match: yay!
        assert!(has_muted_word(
            &[muted_word(
                "yay!",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: yay
        assert!(has_muted_word(
            &[muted_word(
                "yay",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // y!ppee!!
    {
        let rt = rich_text_with_detect_facets("We're federating, y!ppee!!").await?;
        // match: y!ppee
        assert!(has_muted_word(
            &[muted_word(
                "y!ppee",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: y!ppee!
        assert!(has_muted_word(
            &[muted_word(
                "y!ppee!",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // apostrophes: Bluesky's
    {
        let rt = rich_text_with_detect_facets("Yay, Bluesky's mutewords work").await?;
        // match: Bluesky's
        assert!(has_muted_word(
            &[muted_word(
                "Bluesky's",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: Bluesky
        assert!(has_muted_word(
            &[muted_word(
                "Bluesky",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: bluesky
        assert!(has_muted_word(
            &[muted_word(
                "bluesky",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: blueskys
        assert!(has_muted_word(
            &[muted_word(
                "blueskys",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // Why so S@assy?
    {
        let rt = rich_text_with_detect_facets("Why so S@assy?").await?;
        // match: S@assy
        assert!(has_muted_word(
            &[muted_word(
                "S@assy",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: s@assy
        assert!(has_muted_word(
            &[muted_word(
                "s@assy",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // New York Times
    {
        let rt = rich_text_with_detect_facets("New York Times").await?;
        // match: new york times
        assert!(has_muted_word(
            &[muted_word(
                "new york times",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // !command
    {
        let rt = rich_text_with_detect_facets("Idk maybe a bot !command").await?;
        // match: !command
        assert!(has_muted_word(
            &[muted_word(
                "!command",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: command
        assert!(has_muted_word(
            &[muted_word(
                "command",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // no match: !command
        let rt = rich_text_with_detect_facets("Idk maybe a bot command").await?;
        assert!(!has_muted_word(
            &[muted_word(
                "!command",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // e/acc
    {
        let rt = rich_text_with_detect_facets("I'm e/acc pilled").await?;
        // match: e/acc
        assert!(has_muted_word(
            &[muted_word(
                "e/acc",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: acc
        assert!(has_muted_word(
            &[muted_word(
                "acc",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // super-bad
    {
        let rt = rich_text_with_detect_facets("I'm super-bad").await?;
        // match: super-bad
        assert!(has_muted_word(
            &[muted_word(
                "super-bad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: super
        assert!(has_muted_word(
            &[muted_word(
                "super",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: bad
        assert!(has_muted_word(
            &[muted_word(
                "bad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: super bad
        assert!(has_muted_word(
            &[muted_word(
                "super bad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: superbad
        assert!(has_muted_word(
            &[muted_word(
                "superbad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // idk_what_this_would_be
    {
        let rt = rich_text_with_detect_facets("Weird post with idk_what_this_would_be").await?;
        // match: idk what this would be
        assert!(has_muted_word(
            &[muted_word(
                "idk what this would be",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // no match: idk what this would be for
        assert!(!has_muted_word(
            &[muted_word(
                "idk what this would be for",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: idk
        assert!(has_muted_word(
            &[muted_word(
                "idk",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: idkwhatthiswouldbe
        assert!(has_muted_word(
            &[muted_word(
                "idkwhatthiswouldbe",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // parentheses
    {
        let rt = rich_text_with_detect_facets("Post with context(iykyk)").await?;
        // match: context(iykyk)
        assert!(has_muted_word(
            &[muted_word(
                "context(iykyk)",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: context
        assert!(has_muted_word(
            &[muted_word(
                "context",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: iykyk
        assert!(has_muted_word(
            &[muted_word(
                "iykyk",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: (iykyk)
        assert!(has_muted_word(
            &[muted_word(
                "(iykyk)",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // 🦋
    {
        let rt = rich_text_with_detect_facets("Post with 🦋").await?;
        // match: 🦋
        assert!(has_muted_word(
            &[muted_word("🦋", MutedWordTarget::Content, ActorTarget::All)],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
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
            &[muted_word(
                "stop worrying",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
        // match: turtles, or how
        assert!(has_muted_word(
            &[muted_word(
                "turtles, or how",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            None,
            None
        ));
    }
    // languages without spaces
    {
        let rt = rich_text_with_detect_facets("私はカメが好きです、またはどのようにして心配するのをやめてインターネットを愛するようになったのか").await?;
        // match: インターネット
        assert!(has_muted_word(
            &[muted_word(
                "インターネット",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            &rt.text,
            rt.facets.as_ref(),
            Some(&vec![]),
            Some(&vec!["ja".parse().expect("invalid lang")]),
            None
        ));
    }
    Ok(())
}

#[test]
fn facet_with_multiple_features() {
    // multiple tags
    {
        assert!(has_muted_word(
            &[muted_word(
                "bad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            "tags",
            Some(&vec![MainData {
                features: vec![
                    Union::Refs(MainFeaturesItem::Tag(Box::new(
                        TagData {
                            tag: String::from("good")
                        }
                        .into()
                    ))),
                    Union::Refs(MainFeaturesItem::Tag(Box::new(
                        TagData {
                            tag: String::from("bad")
                        }
                        .into()
                    )))
                ],
                index: ByteSliceData {
                    byte_end: 4,
                    byte_start: 0,
                }
                .into()
            }
            .into()]),
            Some(&vec![]),
            None,
            None
        ))
    }
    // other features
    {
        assert!(has_muted_word(
            &[muted_word(
                "bad",
                MutedWordTarget::Content,
                ActorTarget::All
            )],
            "test",
            Some(&vec![MainData {
                features: vec![
                    Union::Unknown(UnknownData {
                        r#type: String::from("com.example.richtext.facet#other"),
                        data: Ipld::Map(BTreeMap::from_iter([(
                            String::from("foo"),
                            Ipld::String(String::from("bar"))
                        ),]))
                    }),
                    Union::Refs(MainFeaturesItem::Tag(Box::new(
                        TagData {
                            tag: String::from("bad")
                        }
                        .into()
                    )))
                ],
                index: ByteSliceData {
                    byte_end: 4,
                    byte_start: 0,
                }
                .into()
            }
            .into()]),
            Some(&vec![]),
            None,
            None
        ))
    }
}

#[test]
fn does_not_mute_own_post() {
    let prefs = &moderation_prefs("words", MutedWordTarget::Content, ActorTarget::All, None);
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
    use atrium_api::types::Unknown;
    use std::ops::DerefMut;

    let prefs = moderation_prefs("words", MutedWordTarget::Tag, ActorTarget::All, None);
    let rt = rich_text_with_detect_facets("Mute #words!").await?;
    let mut post = post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        &rt.text,
        None,
    );
    if let Unknown::Other(data) = &mut post.record {
        if let Ipld::Map(m) = data.deref_mut() {
            if let Some(facets) = rt.facets {
                m.insert(
                    "facets".to_string(),
                    ipld_core::serde::to_ipld(facets).expect("failed to serialize facets"),
                );
            }
        }
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

#[test]
fn timed_mute_words() {
    // non-expired word
    {
        let now = chrono::Utc::now().fixed_offset();
        let prefs = &moderation_prefs(
            "words",
            MutedWordTarget::Content,
            ActorTarget::All,
            Some(Datetime::new(now + Duration::from_secs(1))),
        );
        let post = &post_view(
            &profile_view_basic("bob.test", Some("Bob"), None),
            "Mute words!",
            None,
        );
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
    }
    // expired word
    {
        let now = chrono::Utc::now().fixed_offset();
        let prefs = &moderation_prefs(
            "words",
            MutedWordTarget::Content,
            ActorTarget::All,
            Some(Datetime::new(now - Duration::from_secs(1))),
        );
        let post = &post_view(
            &profile_view_basic("bob.test", Some("Bob"), None),
            "Mute words!",
            None,
        );
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            prefs.clone(),
            HashMap::new(),
        );
        let result = moderator.moderate_post(post);
        assert!(
            !result.ui(DecisionContext::ContentList).filter(),
            "post should not be filtered"
        );
    }
}

#[test]
fn actor_based_mute_words() {
    let prefs = moderation_prefs(
        "words",
        MutedWordTarget::Content,
        ActorTarget::ExcludeFollowing,
        None,
    );
    // followed actor
    {
        let mut author = profile_view_basic("bob.test", Some("Bob"), None);
        author.viewer = Some(viewer_state(Some(String::from("true"))));
        if let Some(viewer) = author.viewer.as_mut() {
            viewer.following = Some(String::from("true"));
        }
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            prefs.clone(),
            HashMap::new(),
        );
        let result = moderator.moderate_post(&post_view(&author, "Mute words!", None));
        assert!(
            !result.ui(DecisionContext::ContentList).filter(),
            "post should not be filtered"
        );
    }
    // non-followed actor
    {
        let mut author = profile_view_basic("carla.test", Some("Carla"), None);
        author.viewer = Some(viewer_state(None));
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            prefs.clone(),
            HashMap::new(),
        );
        let result = moderator.moderate_post(&post_view(&author, "Mute words!", None));
        assert!(
            result.ui(DecisionContext::ContentList).filter(),
            "post should be filtered"
        );
    }
}
