pub mod as_string {
    use std::{borrow::Cow, fmt::Display, str::FromStr};

    use serde::{self, Deserialize, Deserializer, Serializer};

    #[expect(dead_code)]
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        let s = Cow::<'_, str>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

pub mod none_as_empty_string {
    use std::{borrow::Cow, fmt::Display, str::FromStr};

    use serde::{self, Deserialize, Deserializer, Serializer};

    #[expect(dead_code)]
    pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        match value {
            Some(ip) => serializer.serialize_str(&ip.to_string()),
            None => serializer.serialize_str(""),
        }
    }

    #[expect(dead_code)]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        match Cow::<'_, str>::deserialize(deserializer)?.as_ref() {
            "" => Ok(None),
            s => s.parse().map(Some).map_err(serde::de::Error::custom),
        }
    }
}

pub mod host_as_str {
    use std::borrow::Cow;

    use serde::{Deserialize, Deserializer, Serializer};
    use url::Host;

    pub fn serialize<S>(host: &Host, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&host.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Host, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        Host::parse(&str).map_err(serde::de::Error::custom)
    }
}

pub mod option_host_as_str {
    use super::*;

    use std::borrow::Cow;

    use serde::{Deserialize, Deserializer, Serializer};
    use url::Host;

    pub fn serialize<S>(host: &Option<Host>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match host {
            Some(h) => host_as_str::serialize(h, serializer),
            None => serializer.serialize_str(""),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Host>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        match str.as_ref() {
            "" => Ok(None),
            s => Host::parse(s).map(Some).map_err(serde::de::Error::custom),
        }
    }
}

pub mod single_or_sequence {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

    pub fn serialize<S, T>(entries: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match entries {
            [single] => single.serialize(serializer),
            sequence => sequence.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SingleOrSequence<T> {
            Single(T),
            Sequence(Vec<T>),
        }

        Ok(match SingleOrSequence::deserialize(deserializer)? {
            SingleOrSequence::Single(single) => vec![single],
            SingleOrSequence::Sequence(sequence) => sequence,
        })
    }
}
