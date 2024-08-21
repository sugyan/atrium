use rand::{CryptoRng, RngCore};

pub fn get_random_values<R>(rng: &mut R) -> [u8; 16]
where
    R: RngCore + CryptoRng,
{
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    bytes
}
