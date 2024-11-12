use base64::prelude::*;
use rand::Rng;

pub const KEY_SIZE: usize = 48;

pub fn gen_base64_key<const N: usize>() -> String {
    let mut rng = rand::thread_rng();
    BASE64_STANDARD.encode(rng.gen::<[u8; N]>())
}
