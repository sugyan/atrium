mod detection;

use crate::agent::config::Config;
use crate::agent::BskyAgentBuilder;
use crate::error::Result;
use atrium_api::app::bsky::richtext::facet::{ByteSlice, Link, MainFeaturesItem, Mention, Tag};
use atrium_api::types::Union;
use atrium_api::xrpc::XrpcClient;
use detection::{detect_facets, FacetFeaturesItem};
use std::cmp::Ordering;
use unicode_segmentation::UnicodeSegmentation;

const PUBLIC_API_ENDPOINT: &str = "https://public.api.bsky.app";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTextSegment {
    text: String,
    facet: Option<atrium_api::app::bsky::richtext::facet::Main>,
}

impl RichTextSegment {
    pub fn new(
        text: impl AsRef<str>,
        facets: Option<atrium_api::app::bsky::richtext::facet::Main>,
    ) -> Self {
        Self {
            text: text.as_ref().into(),
            facet: facets,
        }
    }
    pub fn mention(&self) -> Option<Mention> {
        self.facet.as_ref().and_then(|facet| {
            facet.features.iter().find_map(|feature| match feature {
                Union::Refs(MainFeaturesItem::Mention(mention)) => Some(mention.as_ref().clone()),
                _ => None,
            })
        })
    }
    pub fn link(&self) -> Option<Link> {
        self.facet.as_ref().and_then(|facet| {
            facet.features.iter().find_map(|feature| match feature {
                Union::Refs(MainFeaturesItem::Link(link)) => Some(link.as_ref().clone()),
                _ => None,
            })
        })
    }
    pub fn tag(&self) -> Option<Tag> {
        self.facet.as_ref().and_then(|facet| {
            facet.features.iter().find_map(|feature| match feature {
                Union::Refs(MainFeaturesItem::Tag(tag)) => Some(tag.as_ref().clone()),
                _ => None,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct RichText {
    text: String,
    facets: Option<Vec<atrium_api::app::bsky::richtext::facet::Main>>,
}

impl RichText {
    const BYTE_SLICE_ZERO: ByteSlice = ByteSlice {
        byte_start: 0,
        byte_end: 0,
    };
    pub fn new(
        text: impl AsRef<str>,
        facets: Option<Vec<atrium_api::app::bsky::richtext::facet::Main>>,
    ) -> Self {
        RichText {
            text: text.as_ref().into(),
            facets,
        }
    }
    pub async fn new_with_detect_facets(
        text: impl AsRef<str>,
        client: impl XrpcClient + Send + Sync,
    ) -> Result<Self> {
        let mut rt = Self {
            text: text.as_ref().into(),
            facets: None,
        };
        rt.detect_facets(client).await?;
        Ok(rt)
    }
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
    pub fn len(&self) -> usize {
        self.text.len()
    }
    pub fn grapheme_len(&self) -> usize {
        self.text.as_str().graphemes(true).count()
    }
    pub fn segments(&self) -> Vec<RichTextSegment> {
        let Some(facets) = self.facets.as_ref() else {
            return vec![RichTextSegment::new(&self.text, None)]
        };
        let mut segments = Vec::new();
        let (mut text_cursor, mut facet_cursor) = (0, 0);
        while facet_cursor < facets.len() {
            let curr_facet = &facets[facet_cursor];
            match text_cursor.cmp(&curr_facet.index.byte_start) {
                Ordering::Less => {
                    segments.push(RichTextSegment::new(
                        &self.text[text_cursor..curr_facet.index.byte_start],
                        None,
                    ));
                }
                Ordering::Greater => {
                    facet_cursor += 1;
                    continue;
                }
                Ordering::Equal => {}
            }
            if curr_facet.index.byte_start < curr_facet.index.byte_end {
                let subtext = &self.text[curr_facet.index.byte_start..curr_facet.index.byte_end];
                if subtext.trim().is_empty() {
                    segments.push(RichTextSegment::new(subtext, None));
                } else {
                    segments.push(RichTextSegment::new(subtext, Some(curr_facet.clone())));
                }
            }
            text_cursor = curr_facet.index.byte_end;
            facet_cursor += 1;
        }
        if text_cursor < self.text.len() {
            segments.push(RichTextSegment::new(&self.text[text_cursor..], None));
        }
        segments
    }
    pub fn insert(&mut self, index: usize, text: impl AsRef<str>) {
        self.text.insert_str(index, text.as_ref());
        if let Some(facets) = self.facets.as_mut() {
            let num_chars_added = text.as_ref().len();
            for facet in facets.iter_mut() {
                // scenario A (before)
                if index <= facet.index.byte_start {
                    facet.index.byte_start += num_chars_added;
                    facet.index.byte_end += num_chars_added;
                }
                // scenario B (inner)
                else if index >= facet.index.byte_start && index < facet.index.byte_end {
                    facet.index.byte_end += num_chars_added;
                }
                // scenario C (after)
                // noop
            }
        }
    }
    pub fn delete(&mut self, start_index: usize, end_index: usize) {
        self.text.drain(start_index..end_index);
        if let Some(facets) = self.facets.as_mut() {
            let num_chars_removed = end_index - start_index;
            for facet in facets.iter_mut() {
                // scenario A (entirely outer)
                if start_index <= facet.index.byte_start && end_index >= facet.index.byte_end {
                    // delete slice (will get removed in final pass)
                    facet.index = Self::BYTE_SLICE_ZERO;
                }
                // scenario B (entirely after)
                else if start_index > facet.index.byte_end {
                    // noop
                }
                // scenario C (partially after)
                else if start_index > facet.index.byte_start
                    && start_index <= facet.index.byte_end
                    && end_index > facet.index.byte_end
                {
                    facet.index.byte_end = start_index;
                }
                // scenario D (entirely inner)
                else if start_index >= facet.index.byte_start && end_index <= facet.index.byte_end
                {
                    facet.index.byte_end -= num_chars_removed;
                }
                // scenario E (partially before)
                else if start_index < facet.index.byte_start
                    && end_index >= facet.index.byte_start
                    && end_index <= facet.index.byte_end
                {
                    facet.index.byte_start = start_index;
                    facet.index.byte_end -= num_chars_removed;
                }
                // scenario F (entirely before)
                else if end_index < facet.index.byte_start {
                    facet.index.byte_start -= num_chars_removed;
                    facet.index.byte_end -= num_chars_removed;
                }
            }
            // filter out any facets that were made irrelevant
            facets.retain(|facet| facet.index.byte_start < facet.index.byte_end);
        }
    }
    pub async fn detect_facets(&mut self, client: impl XrpcClient + Send + Sync) -> Result<()> {
        let agent = BskyAgentBuilder::default()
            .client(client)
            .config(Config {
                endpoint: PUBLIC_API_ENDPOINT.into(),
                ..Default::default()
            })
            .build()
            .await?;
        let facets_without_resolution = detect_facets(&self.text);
        self.facets = if facets_without_resolution.is_empty() {
            None
        } else {
            let mut facets = Vec::new();
            for facet_without_resolution in facets_without_resolution {
                let mut features = Vec::new();
                for feature in facet_without_resolution.features {
                    match feature {
                        FacetFeaturesItem::Mention(mention) => {
                            let did = agent.api.com.atproto.identity.resolve_handle(
                                atrium_api::com::atproto::identity::resolve_handle::Parameters {
                                    handle: mention.handle.parse().expect("invalid handle"),
                                }
                            ).await?.did;
                            features.push(Union::Refs(MainFeaturesItem::Mention(Box::new(
                                Mention { did },
                            ))));
                        }
                        FacetFeaturesItem::Link(link) => {
                            features.push(Union::Refs(MainFeaturesItem::Link(link)));
                        }
                        FacetFeaturesItem::Tag(tag) => {
                            features.push(Union::Refs(MainFeaturesItem::Tag(tag)));
                        }
                    }
                }
                facets.push(atrium_api::app::bsky::richtext::facet::Main {
                    features,
                    index: facet_without_resolution.index,
                });
            }
            Some(facets)
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests;
