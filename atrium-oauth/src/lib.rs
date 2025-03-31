#![doc = include_str!("../README.md")]
mod atproto;
mod constants;
mod error;
mod http_client;
mod jose;
mod keyset;
mod oauth_client;
mod oauth_session;
mod resolver;
mod server_agent;
pub mod store;
mod types;
mod utils;

pub use atproto::{
    AtprotoClientMetadata, AtprotoLocalhostClientMetadata, AuthMethod, GrantType, KnownScope, Scope,
};
pub use error::{Error, Result};
#[cfg(feature = "default-client")]
pub use http_client::default::DefaultHttpClient;
pub use http_client::dpop::DpopClient;
pub use oauth_client::{OAuthClient, OAuthClientConfig};
pub use oauth_session::OAuthSession;
pub use resolver::OAuthResolverConfig;
pub use types::{
    AuthorizeOptionPrompt, AuthorizeOptions, CallbackParams, OAuthClientMetadata, TokenSet,
};

#[cfg(test)]
mod tests {
    use crate::{
        resolver::OAuthResolver,
        types::{
            OAuthAuthorizationServerMetadata, OAuthClientMetadata, OAuthProtectedResourceMetadata,
            TryIntoOAuthClientMetadata,
        },
        AtprotoLocalhostClientMetadata, OAuthResolverConfig,
    };
    use atrium_api::{
        did_doc::{DidDocument, Service},
        types::string::{Did, Handle},
    };
    use atrium_common::resolver::Resolver;
    use atrium_xrpc::HttpClient;
    use jose_jwk::Key;
    use std::sync::Arc;

    pub struct MockDidResolver;

    impl Resolver for MockDidResolver {
        type Input = Did;
        type Output = DidDocument;
        type Error = atrium_identity::Error;
        async fn resolve(&self, did: &Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(DidDocument {
                context: None,
                id: did.as_ref().to_string(),
                also_known_as: None,
                verification_method: None,
                service: Some(vec![Service {
                    id: String::from("#atproto_pds"),
                    r#type: String::from("AtprotoPersonalDataServer"),
                    service_endpoint: String::from("https://aud.example.com"),
                }]),
            })
        }
    }

    pub struct NoopHandleResolver;

    impl Resolver for NoopHandleResolver {
        type Input = Handle;
        type Output = Did;
        type Error = atrium_identity::Error;
        async fn resolve(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
            unimplemented!()
        }
    }

    pub fn oauth_resolver<T>(
        http_client: Arc<T>,
    ) -> OAuthResolver<T, MockDidResolver, NoopHandleResolver>
    where
        T: HttpClient + Send + Sync,
    {
        OAuthResolver::new(
            OAuthResolverConfig {
                did_resolver: MockDidResolver,
                handle_resolver: NoopHandleResolver,
                authorization_server_metadata: Default::default(),
                protected_resource_metadata: Default::default(),
            },
            http_client,
        )
    }

    pub fn dpop_key() -> Key {
        serde_json::from_str(
            r#"{
                "kty": "EC",
                "crv": "P-256",
                "x": "NIRNgPVAwnVNzN5g2Ik2IMghWcjnBOGo9B-lKXSSXFs",
                "y": "iWF-Of43XoSTZxcadO9KWdPTjiCoviSztYw7aMtZZMc",
                "d": "9MuCYfKK4hf95p_VRj6cxKJwORTgvEU3vynfmSgFH2M"
            }"#,
        )
        .expect("key should be valid")
    }

    pub fn server_metadata() -> OAuthAuthorizationServerMetadata {
        OAuthAuthorizationServerMetadata {
            issuer: String::from("https://iss.example.com"),
            token_endpoint: String::from("https://iss.example.com/token"),
            token_endpoint_auth_methods_supported: Some(vec![
                String::from("none"),
                String::from("private_key_jwt"),
            ]),
            ..Default::default()
        }
    }

    pub fn client_metadata() -> OAuthClientMetadata {
        AtprotoLocalhostClientMetadata::default()
            .try_into_client_metadata(&None)
            .expect("client metadata should be valid")
    }

    pub fn protected_resource_metadata() -> OAuthProtectedResourceMetadata {
        OAuthProtectedResourceMetadata {
            resource: String::from("https://aud.example.com"),
            authorization_servers: Some(vec![String::from("https://iss.example.com")]),
            ..Default::default()
        }
    }
}
