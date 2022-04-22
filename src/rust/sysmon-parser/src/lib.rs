//! # (Unofficial) Sysmon XML parser
//!
//! This parses Sysmon events from Windows Events XML. Events from [Sysmon for Linux] have been
//! tested against the library, which appear to have similar schemas.
//!
//! Sysmon schemas used for creating this libary can be found the in `/etc` directory. These were
//! provided by Sysmon (ex: `Sysmon64.exe -s`).
//!
//! This version has been tested against events from the following versions of Sysmon:
//! - System Monitor v13.33 (Windows)
//! - Sysmon For Linux v1.0.0 (Linux)
//!
//! # Event types
//!
//! Not all events types from Sysmon are currently supported. For such events the [`System`] will
//! still parse, but the [`SysmonEvent::event_data`] will be the [`EventData::Unsupported`]
//! variant. The types that currently are supported are:
//!
//!   - [FileCreate]
//!   - [NetworkConnect]
//!   - [ProcessCreate]
//!   - [ProcessTerminate]
//!
//! # Data types
//!
//! Different versions of Sysmon will different schemas for event data. To handle these different
//! event versions, this library uses the [`std::option::Option`] type for fields that are not
//! common across all versions. This will of course present an issue for future versions that
//! decide to drop a field, requiring breaking changes. Instead of putting all field behind the
//! [`std::option::Option`] type to account for this future versions of this library will provide a
//! serde interface, which will let users define the data types they'd like to deserialize to.
//!
//! Sysmon appears to use a sentinel value `-` for fields where data is not present. Instead of
//! attempting to interpret such meaning from these values by using the [`std::option::Option`]
//! type, this will make no such assumptions about the data, and will return the value from Sysmon.
//!
//! # Known issues
//!
//! 1. [xmlparser] is used for parsing the input XML, which supports XML 1.0 only, and verifies the
//! XML adheres the to 1.0 spec. However, Sysmon _may_ include characters that are not valid for
//! the XML 1.0 spec. This includes control characters, which can show up in command line strings
//! on Linux.
//!
//!
//! [Sysmon for Linux]: https://github.com/Sysinternals/SysmonForLinux
//! [xmlparser]: https://github.com/RazrFalcon/xmlparser
//! [FileCreate]: event_data::FileCreateEventData
//! [NetworkConnect]: event_data::NetworkConnectionEventData
//! [ProcessCreate]: event_data::ProcessCreateEventData
//! [ProcessTerminate]: event_data::ProcessTerminatedEventData
//! [Unsupported]: EventData::UnsupportedEventData

#![allow(
    // `from_str` is standard for parsers (ex: [`serde_json::from_str`])
    clippy::should_implement_trait,
    // disable this to keep all matching consistent in the parser
    clippy::single_match,
    clippy::large_enum_variant,
)]

mod error;
mod event;
mod events;
mod util;

use events::SysmonEvents;

pub mod event_data;
pub mod system;

#[doc(inline)]
pub use event_data::EventData;

pub use crate::error::{
    Error,
    Result,
};
#[doc(inline)]
pub use crate::event::SysmonEvent;
#[doc(inline)]
pub use crate::system::System;

/// An iterator over results of parsed Sysmon XML events found in this string slice.
///
/// Unsupported Sysmon events types will result in `event_data: EventData::Unsupported`
///
/// # Example
///
/// ```
/// use sysmon_parser::{SysmonEvent, Result};
///
/// let xml = r#"
/// <Event>
///   <System>
///     <Provider Name="Linux-Sysmon" Guid="{ff032593-a8d3-4f13-b0d6-02dc615a6f97}"/>
///     <EventID>5</EventID>
///     <Version>3</Version>
///     <Level>4</Level>
///     <Task>5</Task>
///     <Opcode>0</Opcode>
///     <Keywords>0x8000000000000000</Keywords>
///     <TimeCreated SystemTime="2022-01-04T19:52:56.313955000Z"/>
///     <EventRecordID>21</EventRecordID>
///     <Correlation/>
///     <Execution ProcessID="49514" ThreadID="49514"/>
///     <Channel>Linux-Sysmon/Operational</Channel>
///     <Computer>hostname</Computer>
///     <Security UserId="0"/>
///   </System>
///   <EventData>
///     <Data Name="RuleName">-</Data>
///     <Data Name="UtcTime">2022-01-04 19:52:56.319</Data>
///     <Data Name="ProcessGuid">{49e2a5f6-a598-61d4-5d5a-d1755b550000}</Data>
///     <Data Name="ProcessId">49529</Data>
///     <Data Name="Image">/usr/bin/id</Data>
///     <Data Name="User">root</Data>
///   </EventData>
/// </Event>"#;
///
/// let events: Vec<Result<SysmonEvent>> = sysmon_parser::parse_events(xml).collect();
/// for event in events {
///     let event: SysmonEvent = event.unwrap();
///     println!("{:#?}", event);
/// }
///
/// ```
pub fn parse_events(input: &str) -> SysmonEvents<'_> {
    SysmonEvents::from(input)
}
