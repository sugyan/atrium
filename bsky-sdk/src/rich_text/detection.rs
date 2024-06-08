use atrium_api::app::bsky::richtext::facet::{ByteSlice, Link, Tag};
use regex::Regex;
use std::sync::OnceLock;

static RE_MENTION: OnceLock<Regex> = OnceLock::new();

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
    facets
}
