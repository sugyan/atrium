mod lexicon;

use lexicon::LexUserType;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::str::FromStr;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct LexiconDoc {
    pub lexicon: u32,
    pub id: String,
    pub revision: Option<u32>,
    pub description: Option<String>,
    pub defs: HashMap<String, LexUserType>,
}

impl FromStr for LexiconDoc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<Self>(s)
    }
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
    fn parse() -> Result<(), Error> {
        let doc = LEXICON_EXAMPLE_TOKEN.parse::<LexiconDoc>()?;
        assert_eq!(doc.lexicon, 1);
        assert_eq!(doc.id, "com.socialapp.actorUser");
        assert_eq!(doc.revision, None);
        assert_eq!(doc.description, None);
        assert_eq!(doc.defs.len(), 1);
        Ok(())
    }
}
