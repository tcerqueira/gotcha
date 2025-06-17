use std::fmt::Display;

use base64::{DecodeError, prelude::*};
use rand::{Rng, RngCore};
use secrecy::Zeroize;
use serde::{Deserialize, Serialize};

pub const KEY_SIZE: usize = 48;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    sqlx::FromRow,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Base64(String);

impl Base64 {
    pub fn random<const N: usize>() -> Self {
        let mut rng = rand::rng();
        Base64(BASE64_STANDARD.encode(rng.random::<[u8; N]>()))
    }

    pub fn random_with<const N: usize>(mut rng: impl RngCore) -> Self {
        Base64(BASE64_STANDARD.encode(rng.random::<[u8; N]>()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Base64 {
    type Error = DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // PERF: find method to just check string validity
        _ = BASE64_STANDARD.decode(value.clone())?;
        Ok(Base64(value))
    }
}

impl secrecy::DebugSecret for Base64 {}

impl Zeroize for Base64 {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    sqlx::FromRow,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Base64UrlSafe(String);

impl Base64UrlSafe {
    pub fn random<const N: usize>() -> Self {
        let mut rng = rand::rng();
        Base64UrlSafe(BASE64_URL_SAFE.encode(rng.random::<[u8; N]>()))
    }

    pub fn random_with<const N: usize>(mut rng: impl RngCore) -> Self {
        Base64UrlSafe(BASE64_URL_SAFE.encode(rng.random::<[u8; N]>()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Base64UrlSafe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for Base64UrlSafe {
    type Error = DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // PERF: find method to just check string validity
        _ = BASE64_URL_SAFE.decode(value.clone())?;
        Ok(Base64UrlSafe(value))
    }
}

impl secrecy::DebugSecret for Base64UrlSafe {}

impl Zeroize for Base64UrlSafe {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
