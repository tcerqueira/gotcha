use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use base64::{DecodeError, prelude::*};
use rand::{Rng, RngCore};
use secrecy::Zeroize;
use serde::{Deserialize, Serialize};

pub const KEY_SIZE: usize = 48;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Standard;
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UrlSafe;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct Base64<A = Standard>(Box<str>, PhantomData<A>);

impl<A> Base64<A> {
    fn new(value: String) -> Self {
        Base64(value.into_boxed_str(), PhantomData)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Base64<Standard> {
    pub fn random<const N: usize>() -> Self {
        let mut rng = rand::rng();
        Self::new(BASE64_STANDARD.encode(rng.random::<[u8; N]>()))
    }

    pub fn random_with<const N: usize>(mut rng: impl RngCore) -> Self {
        Self::new(BASE64_STANDARD.encode(rng.random::<[u8; N]>()))
    }
}

impl Base64<UrlSafe> {
    pub fn random<const N: usize>() -> Self {
        let mut rng = rand::rng();
        Self::new(BASE64_URL_SAFE.encode(rng.random::<[u8; N]>()))
    }

    pub fn random_with<const N: usize>(mut rng: impl RngCore) -> Self {
        Self::new(BASE64_URL_SAFE.encode(rng.random::<[u8; N]>()))
    }
}

impl TryFrom<String> for Base64<Standard> {
    type Error = DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // PERF: find method to just check string validity
        let mut out_buf = [0; KEY_SIZE];
        BASE64_STANDARD.decode_slice_unchecked(&value, &mut out_buf)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<String> for Base64<UrlSafe> {
    type Error = DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // PERF: find method to just check string validity
        let mut out_buf = [0; KEY_SIZE];
        BASE64_URL_SAFE.decode_slice_unchecked(&value, &mut out_buf)?;
        Ok(Self::new(value))
    }
}

impl Debug for Base64<Standard> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Base64<Standard>").field(&self.0).finish()
    }
}

impl Debug for Base64<UrlSafe> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Base64<UrlSafe>").field(&self.0).finish()
    }
}

impl Display for Base64<UrlSafe> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<A> secrecy::DebugSecret for Base64<A> {}

impl<A> Zeroize for Base64<A> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
