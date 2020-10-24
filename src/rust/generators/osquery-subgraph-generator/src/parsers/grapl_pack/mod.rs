use serde::{Serialize, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

pub mod processes;
mod files;
mod process_files;

fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}