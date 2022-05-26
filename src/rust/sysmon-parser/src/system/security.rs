use std::borrow::Cow;

use derive_into_owned::IntoOwned;

/// Identifies the user that logged the event.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-security-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Security<'a> {
    /// The security identifier (SID) of the user in string form.
    pub user_id: Option<Cow<'a, str>>,
}
