use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::{rngs::ThreadRng, CryptoRng, RngCore};
use std::cmp::Ordering;

pub fn generate_nonce() -> String {
    URL_SAFE_NO_PAD.encode(get_random_values::<_, 16>(&mut ThreadRng::default()))
}

pub fn get_random_values<R, const LEN: usize>(rng: &mut R) -> [u8; LEN]
where
    R: RngCore + CryptoRng,
{
    let mut bytes = [0u8; LEN];
    rng.fill_bytes(&mut bytes);
    bytes
}

// 256K > ES (256 > 384 > 512) > PS (256 > 384 > 512) > RS (256 > 384 > 512) > other (in original order)
pub fn compare_algos(a: &String, b: &String) -> Ordering {
    if a == "ES256K" {
        return Ordering::Less;
    }
    if b == "ES256K" {
        return Ordering::Greater;
    }
    for prefix in ["ES", "PS", "RS"] {
        if let Some(stripped_a) = a.strip_prefix(prefix) {
            if let Some(stripped_b) = b.strip_prefix(prefix) {
                if let (Ok(len_a), Ok(len_b)) =
                    (stripped_a.parse::<u32>(), stripped_b.parse::<u32>())
                {
                    return len_a.cmp(&len_b);
                }
            } else {
                return Ordering::Less;
            }
        } else if b.starts_with(prefix) {
            return Ordering::Greater;
        }
    }
    Ordering::Equal
}
