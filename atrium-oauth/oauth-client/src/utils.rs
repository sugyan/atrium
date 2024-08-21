use rand::{CryptoRng, RngCore};

pub fn get_random_values<R, const LEN: usize>(rng: &mut R) -> [u8; LEN]
where
    R: RngCore + CryptoRng,
{
    let mut bytes = [0u8; LEN];
    rng.fill_bytes(&mut bytes);
    bytes
}
