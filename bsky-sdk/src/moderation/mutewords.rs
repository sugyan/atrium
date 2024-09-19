//! Muteword checking logic.
use atrium_api::app::bsky::actor::defs::{MutedWord, ProfileViewBasic};
use atrium_api::app::bsky::richtext::facet;
use atrium_api::types::{string::Language, Union};
use regex::Regex;
use std::sync::OnceLock;

static RE_SPACE_OR_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_WORD_BOUNDARY: OnceLock<Regex> = OnceLock::new();
static RE_LEADING_TRAILING_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_INTERNAL_PUNCTUATION: OnceLock<Regex> = OnceLock::new();

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

/// Check if a text of facets and outline tags contains a muted word.
pub fn has_muted_word(
    muted_words: &[MutedWord],
    text: &str,
    facets: Option<&Vec<facet::Main>>,
    outline_tags: Option<&Vec<String>>,
    langs: Option<&Vec<Language>>,
    actor: Option<&ProfileViewBasic>,
) -> bool {
    let exception = langs
        .as_ref()
        .and_then(|langs| langs.first())
        .map_or(false, |lang| LANGUAGE_EXCEPTIONS.contains(&lang.as_ref().as_str()));
    let mut tags = Vec::new();
    if let Some(outline_tags) = outline_tags {
        tags.extend(outline_tags.iter().map(|t| t.to_lowercase()));
    }
    if let Some(facets) = facets {
        tags.extend(
            facets
                .iter()
                .flat_map(|facet| {
                    facet.features.iter().filter_map(|feature| {
                        if let Union::Refs(facet::MainFeaturesItem::Tag(tag)) = feature {
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

        // check if expired
        if let Some(expires_at) = &mute.expires_at {
            if expires_at.as_ref() < &chrono::Utc::now().fixed_offset() {
                continue;
            }
        }
        // check if actor target
        if let Some(actor_target) = &mute.actor_target {
            if actor_target == "exclude-following"
                && actor
                    .and_then(|actor| {
                        actor.viewer.as_ref().and_then(|viewer| viewer.following.as_ref())
                    })
                    .is_some()
            {
                continue;
            }
        }

        // `content` applies to tags as well
        if tags.contains(&muted_word) {
            return true;
        }
        // rest of the checks are for `content` only
        if !mute.targets.contains(&String::from("content")) {
            continue;
        }
        // single character or other exception, has to use includes
        if (muted_word.chars().count() == 1 || exception) && post_text.contains(&muted_word) {
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
                    .replace_all(&word_trimmed_punctuation, " ")
                    .to_lowercase();
                if spaced_word == muted_word {
                    return true;
                }

                let contiguous_word = spaced_word.replace(char::is_whitespace, "");
                if contiguous_word == muted_word {
                    return true;
                }

                let word_parts =
                    re_internal_punctuation.split(&word_trimmed_punctuation).collect::<Vec<_>>();
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
