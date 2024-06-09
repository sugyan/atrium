use atrium_api::app::bsky::richtext::facet::{ByteSlice, Link, Tag};
use psl;
use regex::Regex;
use std::sync::OnceLock;

static RE_MENTION: OnceLock<Regex> = OnceLock::new();
static RE_URL: OnceLock<Regex> = OnceLock::new();
static RE_ENDING_PUNCTUATION: OnceLock<Regex> = OnceLock::new();

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
                index: ByteSlice {
                    byte_end: m.end(),
                    byte_start: m.start() - 1,
                },
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
            let mut index = ByteSlice {
                byte_end: m.end(),
                byte_start: m.start(),
            };
            // strip ending puncuation
            if RE_ENDING_PUNCTUATION
                .get_or_init(|| Regex::new(r"[.,;:!?]$").expect("invalid regex"))
                .is_match(&uri)
            {
                uri.pop();
                index.byte_end -= 1;
            }
            facets.push(FacetWithoutResolution {
                features: vec![FacetFeaturesItem::Link(Box::new(Link { uri }))],
                index,
            });
        }
    }
    facets
}
