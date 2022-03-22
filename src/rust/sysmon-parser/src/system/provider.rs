use std::borrow::Cow;

use derive_into_owned::IntoOwned;

/// Identifies the provider that logged the event.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-provider-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(
    feature = "serde",
    derive(serde_crate::Serialize, serde_crate::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Provider<'a> {
    /// The name of the event provider that logged the event.
    pub name: Option<Cow<'a, str>>,
    /// The globally unique identifier that uniquely identifies the provider.
    pub guid: Option<uuid::Uuid>,
    /// The name of the event source that published the event (if the event source is from the legacy Event Logging API).
    pub event_source_name: Option<Cow<'a, str>>,
}
