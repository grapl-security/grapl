use std::{
    fmt::Display,
    str::FromStr,
};

use serde::{
    Deserialize,
    Deserializer,
};

pub mod files;
pub mod process_files;
pub mod processes;

fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
