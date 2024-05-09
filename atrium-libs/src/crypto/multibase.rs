use super::error::Result;

pub fn multibase_to_bytes(mb: &str) -> Result<Vec<u8>> {
    let (_, bytes) = multibase::decode(mb)?;
    Ok(bytes)
}
