use chrono::{
    DateTime,
    Utc,
};

/// The time stamp that identifies when the event was logged.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-timecreated-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct TimeCreated {
    /// The system time of when the event was logged.
    pub system_time: DateTime<Utc>,
}
