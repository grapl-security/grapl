/// The identifier that the provider used to identify the event.
///
/// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum EventId {
    ProcessCreation,
    ProcessChangedFileCreationTime,
    NetworkConnection,
    SysmonServiceStateChange,
    ProcessTerminated,
    DriverLoaded,
    ImageLoaded,
    CreateRemoteThread,
    RawAccessRead,
    ProcessAccess,
    FileCreate,
    RegistryCreateOrDelete,
    RegistryValueSet,
    RegistryKeyValueRename,
    FileCreateStreamHash,
    ServiceConfigurationChange,
    PipeCreated,
    PipeConnected,
    WmiEventFilter,
    WmiEventConsumer,
    WmiEventConsumerToFilter,
    DnsQuery,
    FileDelete,
    ClipboardChange,
    ProcessTampering,
    FileDeleteDetected,
    Error,
    Unknown,
}

impl std::str::FromStr for EventId {
    type Err = std::convert::Infallible;

    /// This provides a mapping between the serialized <EventId> and Sysmon events.
    ///
    /// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events>
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "1" => EventId::ProcessCreation,
            "2" => EventId::ProcessChangedFileCreationTime,
            "3" => EventId::NetworkConnection,
            "4" => EventId::SysmonServiceStateChange,
            "5" => EventId::ProcessTerminated,
            "6" => EventId::DriverLoaded,
            "7" => EventId::ImageLoaded,
            "8" => EventId::CreateRemoteThread,
            "9" => EventId::RawAccessRead,
            "10" => EventId::ProcessAccess,
            "11" => EventId::FileCreate,
            "12" => EventId::RegistryCreateOrDelete,
            "13" => EventId::RegistryValueSet,
            "14" => EventId::RegistryKeyValueRename,
            "15" => EventId::FileCreateStreamHash,
            "16" => EventId::ServiceConfigurationChange,
            "17" => EventId::PipeCreated,
            "18" => EventId::PipeConnected,
            "19" => EventId::WmiEventFilter,
            "20" => EventId::WmiEventConsumer,
            "21" => EventId::WmiEventConsumerToFilter,
            "22" => EventId::DnsQuery,
            "23" => EventId::FileDelete,
            "24" => EventId::ClipboardChange,
            "25" => EventId::ProcessTampering,
            "26" => EventId::FileDeleteDetected,
            "255" => EventId::Error,
            _ => EventId::Unknown,
        };

        Ok(result)
    }
}
