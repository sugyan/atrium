use super::{post_view, profile_view_basic};
use crate::moderation::decision::DecisionContext;
use crate::moderation::{ModerationPrefs, Moderator};
use atrium_api::app::bsky::actor::defs::MutedWord;
use std::collections::HashMap;

// TODO: RichText

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
