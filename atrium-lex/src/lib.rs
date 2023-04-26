pub mod lexicon;

use lexicon::LexUserType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Lexicon {
    Lexicon1 = 1,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LexiconDoc {
    pub lexicon: Lexicon,
    pub id: String,
    pub revision: Option<u32>,
    pub description: Option<String>,
    pub defs: HashMap<String, LexUserType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEXICON_EXAMPLE_TOKEN: &str = r#"
{
  "lexicon": 1,
  "id": "com.socialapp.actorUser",
  "defs": {
    "main": {
      "type": "token",
      "description": "Actor type of 'User'"
    }
  }
}"#;

    #[test]
    fn parse() {
        let doc = serde_json::from_str::<LexiconDoc>(LEXICON_EXAMPLE_TOKEN)
            .expect("failed to deserialize");
        assert_eq!(doc.lexicon, Lexicon::Lexicon1);
        assert_eq!(doc.id, "com.socialapp.actorUser");
        assert_eq!(doc.revision, None);
        assert_eq!(doc.description, None);
        assert_eq!(doc.defs.len(), 1);
    }
}
