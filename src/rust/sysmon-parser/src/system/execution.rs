/// Contains information about the process and thread that logged the event.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-execution-systempropertiestype-element>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Execution {
    /// Identifies the process that generated the event.
    pub process_id: u32,
    /// Identifies the thread that generated the event.
    pub thread_id: u32,
    /// The identification number for the processor that processed the event.
    pub processor_id: Option<u8>,
    /// The identification number for the terminal server session in which the event occurred.
    pub session_id: Option<u32>,
    /// Elapsed execution time for kernel-mode instructions, in CPU time units.
    pub kernel_time: Option<u32>,
    /// Elapsed execution time for user-mode instructions, in CPU time units.
    pub user_time: Option<u32>,
    /// For ETW private sessions, the elapsed execution time for user-mode instructions, in CPU ticks.
    pub processor_time: Option<u32>,
}
