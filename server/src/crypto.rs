use base64::prelude::*;
use rand::Rng;

pub const KEY_SIZE: usize = 48;

pub fn gen_base64_key<const N: usize>() -> String {
    let mut rng = rand::rng();
    BASE64_STANDARD.encode(rng.random::<[u8; N]>())
}

pub fn gen_base64_url_safe_key<const N: usize>() -> String {
    let mut rng = rand::rng();
    BASE64_URL_SAFE.encode(rng.random::<[u8; N]>())
}
