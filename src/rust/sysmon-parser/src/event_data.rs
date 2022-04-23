use derive_into_owned::IntoOwned;

mod file_create;
mod file_create_stream_hash;
mod network_connect;
mod process_creation;
mod process_terminated;

pub use file_create::FileCreateEventData;
pub use file_create_stream_hash::FileCreateStreamHashEventData;
pub use network_connect::NetworkConnectionEventData;
pub use process_creation::ProcessCreateEventData;
pub use process_terminated::ProcessTerminatedEventData;

pub const UTC_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Event-specific data for the Sysmon event. This inludes the data found in the `<EventData>` element.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventData<'a> {
    /// Event ID 11: FileCreate
    ///
    /// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-11-filecreate>
    FileCreate(FileCreateEventData<'a>),

    /// Event ID 15: FileCreateStreamHash
    ///
    /// https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-15-filecreatestreamhash
    FileCreateStreamHash(FileCreateStreamHashEventData<'a>),

    /// Event ID 3: Network connection
    ///
    /// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-3-network-connection>
    NetworkConnect(NetworkConnectionEventData<'a>),
    /// Event ID 1: Process creation
    ///
    /// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-1-process-creation>
    ProcessCreate(ProcessCreateEventData<'a>),

    /// Event ID 5: Process terminated
    ///
    /// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-5-process-terminated>
    ProcessTerminate(ProcessTerminatedEventData<'a>),

    /// Unsupported event type
    Unsupported,
}
