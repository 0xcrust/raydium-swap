use serde::de::{Deserialize, Deserializer, Error};
use serde::{Serialize, Serializer};
use std::str::FromStr;

pub mod field_as_string {
    use super::*;
    pub fn serialize<T, S>(t: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ToString,
        S: Serializer,
    {
        t.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        D: Deserializer<'de>,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let s: String = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|e| Error::custom(format!("Parse error: {:?}", e)))
    }
}

pub mod option_field_as_string {
    use super::*;
    pub fn serialize<T, S>(t: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ToString,
        S: Serializer,
    {
        if let Some(t) = t {
            t.to_string().serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        D: Deserializer<'de>,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let t: Option<String> = Option::deserialize(deserializer)?;

        match t {
            Some(s) => T::from_str(&s)
                .map(Some)
                .map_err(|_| Error::custom(format!("Parse error for {}", s))),
            None => Ok(None),
        }
    }
}
