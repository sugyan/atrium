use atrium_api::types::string::Did;

use crate::server_agent::OAuthServerAgent;

pub struct OAuthSession<C, D, H> {
    pub server: OAuthServerAgent<crate::DefaultHttpClient, D, H>,
    pub sub: Did,
    // private readonly sessionGetter: SessionGetter,
    session_cache: C,
    // fetch: Fetch = globalThis.fetch,
    // pub dpop_client: C,
}

impl<C, D, H> OAuthSession<C, D, H> {
    pub fn new(
        server: OAuthServerAgent<crate::DefaultHttpClient, D, H>,
        sub: Did,
        session_cache: C,
    ) -> Self {
        Self { server, sub, session_cache }
    }
}
