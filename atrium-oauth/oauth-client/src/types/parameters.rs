use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
    pub iss: Option<String>,
}
