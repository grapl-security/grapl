#[cfg(feature = "serde")]
use chrono::serde::ts_milliseconds;
use chrono::{
    DateTime,
    Utc,
};

/// The time stamp that identifies when the event was logged.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-timecreated-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeCreated {
    /// The system time of when the event was logged.
    #[cfg_attr(feature = "serde", serde(with = "ts_milliseconds"))]
    pub system_time: DateTime<Utc>,
}
