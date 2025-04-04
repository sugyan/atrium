use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationResponseType {
    Code,
    Token,
    // OIDC (https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html)
    IdToken,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationResponseMode {
    Query,
    Fragment,
    // https://openid.net/specs/oauth-v2-form-post-response-mode-1_0.html#FormPostResponseMode
    FormPost,
}

#[derive(Serialize, Deserialize)]
pub enum AuthorizationCodeChallengeMethod {
    S256,
    #[serde(rename = "plain")]
    Plain,
}

#[derive(Serialize, Deserialize)]
pub struct PushedAuthorizationRequestParameters {
    // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.1
    pub response_type: AuthorizationResponseType,
    pub redirect_uri: String,
    pub state: String,
    pub scope: Option<String>,
    // https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html#ResponseModes
    pub response_mode: Option<AuthorizationResponseMode>,
    // https://datatracker.ietf.org/doc/html/rfc7636#section-4.3
    pub code_challenge: String,
    pub code_challenge_method: AuthorizationCodeChallengeMethod,
    // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest
    pub login_hint: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenGrantType {
    AuthorizationCode,
    RefreshToken,
}

#[derive(Serialize, Deserialize)]
pub struct TokenRequestParameters {
    // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3
    pub grant_type: TokenGrantType,
    pub code: String,
    pub redirect_uri: String,
    // https://datatracker.ietf.org/doc/html/rfc7636#section-4.5
    pub code_verifier: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshRequestParameters {
    // https://datatracker.ietf.org/doc/html/rfc6749#section-6
    pub grant_type: TokenGrantType,
    pub refresh_token: String,
    pub scope: Option<String>,
}

// https://datatracker.ietf.org/doc/html/rfc7009#section-2.1
#[derive(Serialize, Deserialize)]
pub struct RevocationRequestParameters {
    pub token: String,
    // ?
    // pub token_type_hint: Option<String>,
}
