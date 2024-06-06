use atrium_api::app::bsky::richtext::facet::ByteSlice;
use unicode_segmentation::UnicodeSegmentation;

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
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
    pub fn len(&self) -> usize {
        self.text.len()
    }
    pub fn grapheme_len(&self) -> usize {
        self.text.as_str().graphemes(true).count()
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
}

#[cfg(test)]
mod tests;
