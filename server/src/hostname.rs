use std::{borrow::Cow, fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::{Host, ParseError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hostname(String);

impl Hostname {
    pub fn parse(host_str: &str) -> Result<Self, ParseError> {
        let host = Host::parse(host_str)?;
        Ok(Hostname::new_unchecked(host.to_string()))
    }

    pub fn new_unchecked(host_str: String) -> Self {
        Hostname(host_str)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for Hostname {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Hostname {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        Hostname::parse(&str).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Hostname {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hostname::parse(s)
    }
}

impl Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
