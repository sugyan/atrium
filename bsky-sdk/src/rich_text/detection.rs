use atrium_api::app::bsky::richtext::facet::{
    ByteSlice, ByteSliceData, Link, LinkData, Tag, TagData,
};
use psl;
use regex::Regex;
use std::sync::OnceLock;

static RE_MENTION: OnceLock<Regex> = OnceLock::new();
static RE_URL: OnceLock<Regex> = OnceLock::new();
static RE_ENDING_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_TRAILING_PUNCTUATION: OnceLock<Regex> = OnceLock::new();
static RE_TAG: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetWithoutResolution {
    pub features: Vec<FacetFeaturesItem>,
    pub index: ByteSlice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetFeaturesItem {
    Mention(Box<MentionWithoutResolution>),
    Link(Box<Link>),
    Tag(Box<Tag>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentionWithoutResolution {
    pub handle: String,
}

pub fn detect_facets(text: &str) -> Vec<FacetWithoutResolution> {
    let mut facets = Vec::new();
    // mentions
    {
        let re = RE_MENTION
            .get_or_init(|| Regex::new(r"(?:^|\s|\()@(([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)\b").expect("invalid regex"));
        for capture in re.captures_iter(text) {
            let Some(m) = capture.get(1) else {
                continue;
            };
            facets.push(FacetWithoutResolution {
                features: vec![FacetFeaturesItem::Mention(Box::new(
                    MentionWithoutResolution {
                        handle: m.as_str().into(),
                    },
                ))],
                index: ByteSliceData {
                    byte_end: m.end(),
                    byte_start: m.start() - 1,
                }
                .into(),
            });
        }
    }
    // links
    {
        let re = RE_URL.get_or_init(|| {
            Regex::new(
                r"(?:^|\s|\()((?:https?:\/\/[\S]+)|(?:(?<domain>[a-z][a-z0-9]*(?:\.[a-z0-9]+)+)[\S]*))",
            )
            .expect("invalid regex")
        });
        for capture in re.captures_iter(text) {
            let m = capture.get(1).expect("invalid capture");
            let mut uri = if let Some(domain) = capture.name("domain") {
                if !psl::suffix(domain.as_str().as_bytes())
                    .map_or(false, |suffix| suffix.is_known())
                {
                    continue;
                }
                format!("https://{}", m.as_str())
            } else {
                m.as_str().into()
            };
            let mut index = ByteSliceData {
                byte_end: m.end(),
                byte_start: m.start(),
            };
            // strip ending puncuation
            if (RE_ENDING_PUNCTUATION
                .get_or_init(|| Regex::new(r"[.,;:!?]$").expect("invalid regex"))
                .is_match(&uri))
                || (uri.ends_with(')') && !uri.contains('('))
            {
                uri.pop();
                index.byte_end -= 1;
            }
            facets.push(FacetWithoutResolution {
                features: vec![FacetFeaturesItem::Link(Box::new(LinkData { uri }.into()))],
                index: index.into(),
            });
        }
    }
    // tags
    {
        let re = RE_TAG.get_or_init(|| {
            Regex::new(
                r"(?:^|\s)([#＃])([^\s\u00AD\u2060\u200A\u200B\u200C\u200D\u20e2]*[^\d\s\p{P}\u00AD\u2060\u200A\u200B\u200C\u200D\u20e2]+[^\s\u00AD\u2060\u200A\u200B\u200C\u200D\u20e2]*)?",
            )
            .expect("invalid regex")
        });
        for capture in re.captures_iter(text) {
            if let Some(tag) = capture.get(2) {
                // strip ending punctuation and any spaces
                let tag = RE_TRAILING_PUNCTUATION
                    .get_or_init(|| Regex::new(r"\p{P}+$").expect("invalid regex"))
                    .replace(tag.as_str(), "");
                // look-around, including look-ahead and look-behind, is not supported in `regex`
                if tag.starts_with('\u{fe0f}') {
                    continue;
                }
                if tag.len() > 64 {
                    continue;
                }
                let leading = capture.get(1).expect("invalid capture");
                let index = ByteSliceData {
                    byte_end: leading.end() + tag.len(),
                    byte_start: leading.start(),
                }
                .into();
                facets.push(FacetWithoutResolution {
                    features: vec![FacetFeaturesItem::Tag(Box::new(
                        TagData { tag: tag.into() }.into(),
                    ))],
                    index,
                });
            }
        }
    }
    facets
}
