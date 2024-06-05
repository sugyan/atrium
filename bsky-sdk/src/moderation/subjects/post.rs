use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectPost};
use super::super::Moderator;
use atrium_api::app::bsky::actor::defs::MutedWord;
use atrium_api::app::bsky::embed::record::{ViewBlocked, ViewRecord, ViewRecordRefs};
use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
use atrium_api::app::bsky::richtext::facet::MainFeaturesItem;
use atrium_api::records::{KnownRecord, Record};
use atrium_api::types::Union;
use regex::Regex;
use std::sync::OnceLock;

static RE_SPACE_OR_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_WORD_BOUNDARY: OnceLock<Regex> = OnceLock::new();
static RE_LEADING_TRAILING_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_INTERNAL_PUNCTUATION: OnceLock<Regex> = OnceLock::new();

impl Moderator {
    pub fn decide_post(&self, subject: &SubjectPost) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        let is_me = self.user_did.as_ref() == Some(&subject.author.did);
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(is_me);
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        if check_hidden_post(subject, &self.prefs.hidden_posts) {
            acc.add_hidden();
        }
        if !is_me && check_muted_words(subject, &self.prefs.muted_words) {
            acc.add_muted_word();
        }

        let embed_acc = match &subject.embed {
            Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(view))) => {
                match &view.record {
                    Union::Refs(ViewRecordRefs::ViewRecord(record)) => {
                        // quoted post
                        Some(self.decide_quoted_post(record))
                    }
                    Union::Refs(ViewRecordRefs::ViewBlocked(blocked)) => {
                        // blocked quote post
                        Some(self.decide_bloked_quoted_post(blocked))
                    }
                    _ => None,
                }
            }
            Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(view))) => {
                match &view.record.record {
                    Union::Refs(ViewRecordRefs::ViewRecord(record)) => {
                        // quoted post with media
                        Some(self.decide_quoted_post(record))
                    }
                    Union::Refs(ViewRecordRefs::ViewBlocked(blocked)) => {
                        // blocked quote post with media
                        Some(self.decide_bloked_quoted_post(blocked))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        let mut decisions = vec![acc];
        if let Some(mut embed_acc) = embed_acc {
            embed_acc.downgrade();
            decisions.push(embed_acc);
        }
        let author = subject.author.clone().into();
        decisions.extend([self.decide_account(&author), self.decide_profile(&author)]);
        ModerationDecision::merge(&decisions)
    }
    fn decide_quoted_post(&self, subject: &ViewRecord) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        ModerationDecision::merge(&[
            acc,
            self.decide_account(&subject.author.clone().into()),
            self.decide_profile(&subject.author.clone().into()),
        ])
    }
    fn decide_bloked_quoted_post(&self, subject: &ViewBlocked) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(viewer) = &subject.author.viewer {
            if viewer.muted.unwrap_or_default() {
                if let Some(list_view) = &viewer.muted_by_list {
                    acc.add_muted_by_list(list_view);
                } else {
                    acc.add_muted();
                }
            }
            if viewer.blocking.is_some() {
                if let Some(list_view) = &viewer.blocking_by_list {
                    acc.add_blocking_by_list(list_view);
                } else {
                    acc.add_blocking();
                }
            }
            if viewer.blocked_by.unwrap_or_default() {
                acc.add_blocked_by();
            }
        }
        acc
    }
}

fn check_hidden_post(subject: &SubjectPost, hidden_posts: &[String]) -> bool {
    if hidden_posts.is_empty() {
        return false;
    }
    if hidden_posts.contains(&subject.uri) {
        return true;
    }
    match &subject.embed {
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(record)) = &view.record {
                if hidden_posts.contains(&record.uri) {
                    return true;
                }
            }
        }
        Some(Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(view))) => {
            if let Union::Refs(ViewRecordRefs::ViewRecord(record)) = &view.record.record {
                if hidden_posts.contains(&record.uri) {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}
fn check_muted_words(subject: &SubjectPost, muted_words: &[MutedWord]) -> bool {
    if muted_words.is_empty() {
        return false;
    }
    let Record::Known(KnownRecord::AppBskyFeedPost(post)) = &subject.record else {
        return false;
    };
    if has_muted_word(
        muted_words,
        &post.text,
        &post.facets,
        &post.tags,
        &post.langs,
    ) {
        return true;
    }

    false
}

/**
 * List of 2-letter lang codes for languages that either don't use spaces, or
 * don't use spaces in a way conducive to word-based filtering.
 *
 * For these, we use a simple `String.includes` to check for a match.
 */
const LANGUAGE_EXCEPTIONS: [&str; 5] = [
    "ja", // Japanese
    "zh", // Chinese
    "ko", // Korean
    "th", // Thai
    "vi", // Vietnamese
];

fn has_muted_word(
    muted_words: &[MutedWord],
    text: &str,
    facets: &Option<Vec<atrium_api::app::bsky::richtext::facet::Main>>,
    outline_tags: &Option<Vec<String>>,
    langs: &Option<Vec<atrium_api::types::string::Language>>,
) -> bool {
    let exception = langs
        .as_ref()
        .and_then(|langs| langs.first())
        .map_or(false, |lang| {
            LANGUAGE_EXCEPTIONS.contains(&lang.as_ref().as_str())
        });
    let mut tags = Vec::new();
    if let Some(outline_tags) = outline_tags {
        tags.extend(outline_tags.iter().map(|t| t.to_lowercase()));
    }
    if let Some(facets) = facets {
        tags.extend(
            facets
                .iter()
                .filter_map(|facet| {
                    facet.features.iter().find_map(|feature| {
                        if let Union::Refs(MainFeaturesItem::Tag(tag)) = feature {
                            Some(&tag.tag)
                        } else {
                            None
                        }
                    })
                })
                .map(|t| t.to_lowercase())
                .collect::<Vec<_>>(),
        )
    }
    for mute in muted_words {
        let muted_word = mute.value.to_lowercase();
        let post_text = text.to_lowercase();
        // `content` applies to tags as well
        if tags.contains(&muted_word) {
            return true;
        }
        // rest of the checks are for `content` only
        if !mute.targets.contains(&String::from("content")) {
            continue;
        }
        // single character or other exception, has to use includes
        if (muted_word.len() == 1 || exception) && post_text.contains(&muted_word) {
            return true;
        }
        // too long
        if muted_word.len() > post_text.len() {
            continue;
        }
        // exact match
        if muted_word == post_text {
            return true;
        }
        // any muted phrase with space or punctuation
        if RE_SPACE_OR_PUNCTUATION
            .get_or_init(|| Regex::new(r"\s|\p{P}").expect("invalid regex"))
            .is_match(&muted_word)
            && post_text.contains(&muted_word)
        {
            return true;
        }

        // check individual character groups
        let words = RE_WORD_BOUNDARY
            .get_or_init(|| Regex::new(r"[\s\n\t\r\f\v]+?").expect("invalid regex"))
            .split(&post_text)
            .collect::<Vec<_>>();
        for word in words {
            if word == muted_word {
                return true;
            }
            // compare word without leading/trailing punctuation, but allow internal
            // punctuation (such as `s@ssy`)
            let word_trimmed_punctuation = RE_LEADING_TRAILING_PUNCTUATION
                .get_or_init(|| Regex::new(r"^\p{P}+|\p{P}+$").expect("invalid regex"))
                .replace_all(word, "");
            if muted_word == word_trimmed_punctuation {
                return true;
            }
            if muted_word.len() > word_trimmed_punctuation.len() {
                continue;
            }

            let re_internal_punctuation = RE_INTERNAL_PUNCTUATION
                .get_or_init(|| Regex::new(r"\p{P}").expect("invalid regex"));
            if re_internal_punctuation.is_match(&word_trimmed_punctuation) {
                let spaced_word = re_internal_punctuation
                    .replace_all(&muted_word, " ")
                    .to_lowercase();
                if spaced_word == muted_word {
                    return true;
                }

                let contiguous_word = spaced_word.replace(char::is_whitespace, "");
                if contiguous_word == muted_word {
                    return true;
                }

                let word_parts = re_internal_punctuation
                    .split(&word_trimmed_punctuation)
                    .collect::<Vec<_>>();
                for word_part in word_parts {
                    if word_part == muted_word {
                        return true;
                    }
                }
            }
        }
    }
    false
}
