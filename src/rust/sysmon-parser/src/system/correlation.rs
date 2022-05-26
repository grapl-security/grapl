/// The activity identifiers that consumers can use to group related events together.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-correlation-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Correlation {
    /// A globally unique identifier that identifies the current activity. The events that are
    /// published with this identifier are part of the same activity.
    pub activity_id: Option<uuid::Uuid>,
    /// A globally unique identifier that identifies the activity to which control was transferred
    /// to. The related events would then have this identifier as their ActivityID identifier.
    pub related_activity_id: Option<uuid::Uuid>,
}
