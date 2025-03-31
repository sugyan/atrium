mod client_metadata;
mod metadata;
mod request;
mod response;
mod token;

pub use self::client_metadata::*;
pub use self::metadata::*;
pub use self::request::*;
pub use self::response::*;
pub use self::token::*;
use crate::atproto::{KnownScope, Scope};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum AuthorizeOptionPrompt {
    Login,
    None,
    Consent,
    SelectAccount,
}

impl From<AuthorizeOptionPrompt> for String {
    fn from(value: AuthorizeOptionPrompt) -> Self {
        match value {
            AuthorizeOptionPrompt::Login => String::from("login"),
            AuthorizeOptionPrompt::None => String::from("none"),
            AuthorizeOptionPrompt::Consent => String::from("consent"),
            AuthorizeOptionPrompt::SelectAccount => String::from("select_account"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeOptions {
    pub redirect_uri: Option<String>,
    pub scopes: Vec<Scope>,
    pub prompt: Option<AuthorizeOptionPrompt>,
    pub state: Option<String>,
}

impl Default for AuthorizeOptions {
    fn default() -> Self {
        Self {
            redirect_uri: None,
            scopes: vec![Scope::Known(KnownScope::Atproto)],
            prompt: None,
            state: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
    pub iss: Option<String>,
}
