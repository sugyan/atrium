/// The URI for the XRPC `WebSocket` connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrpcUri<'a> {
  base_uri: &'a str,
  nsid: &'a str,
}
impl<'a> XrpcUri<'a> {
  pub const fn new(base_uri: &'a str, nsid: &'a str) -> Self {
    Self { base_uri, nsid }
  }

  pub fn to_uri(&self) -> String {
    let XrpcUri { base_uri, nsid } = self;
    format!("wss://{base_uri}/xrpc/{nsid}")
  }
}
