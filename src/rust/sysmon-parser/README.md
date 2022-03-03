# (Unofficial) Sysmon parser

This is an unofficial
[Sysmon](https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon) parser
for Rust that provides type definitions for a number of Sysmon event types and
deserialization routines for Windows Event XML.

This intends to serve as a replacement for the
[sysmon](https://crates.io/crates/sysmon) crate. The new name should help
clarify the scope of crate's purpose.

## Support

This version has been tested against events from the following versions of
Sysmon:

- System Monitor v13.33 (Windows)
- Sysmon For Linux v1.0.0 (Linux)

### Event types

Not all event types are currently supported. Support for more types is planned
for future versions. The event types currently supported are:

- FileCreate
- NetworkConnect
- ProcessCreate
- ProcessTerminate

## Example

### Single event

```rust
use sysmon_parser::{SysmonEvent, Result};

let xml = r#"
  <Event>
    <System>
      <Provider Name="Linux-Sysmon" Guid="{ff032593-a8d3-4f13-b0d6-02dc615a6f97}"/>
      <EventID>5</EventID>
      <Version>3</Version>
      <Level>4</Level>
      <Task>5</Task>
      <Opcode>0</Opcode>
      <Keywords>0x8000000000000000</Keywords>
      <TimeCreated SystemTime="2022-01-04T19:52:56.313955000Z"/>
      <EventRecordID>21</EventRecordID>
      <Correlation/>
      <Execution ProcessID="49514" ThreadID="49514"/>
      <Channel>Linux-Sysmon/Operational</Channel>
      <Computer>hostname</Computer>
      <Security UserId="0"/>
    </System>
    <EventData>
      <Data Name="RuleName">-</Data>
      <Data Name="UtcTime">2022-01-04 19:52:56.319</Data>
      <Data Name="ProcessGuid">{49e2a5f6-a598-61d4-5d5a-d1755b550000}</Data>
      <Data Name="ProcessId">49529</Data>
      <Data Name="Image">/usr/bin/id</Data>
      <Data Name="User">root</Data>
    </EventData>
  </Event>"#;

let process_creation_event = sysmon_parser::SysmonEvent::from_str(&xml)?;
```

### Multiple events

`sysmon_parser::parse_events` produces an iterator for parsing multiple events.

```rust
for event in sysmon_parser::parse_events(xml) {
    ...
}
```

## Known issues

1. [xmlparser](https://github.com/RazrFalcon/xmlparser) is used for parsing the
   input XML, which supports XML 1.0 only, and verifies the XML adheres the to
   1.0 spec. However, Sysmon _may_ include characters that are not valid for the
   XML 1.0 spec. This includes control characters, which can show up in command
   line strings on Linux.

2. The error reporting needs improvement. For parsing errors, like parsing ints,
   the errors at the moment only report that there was a ParseIntError, but it
   doesn't include the other helpful information like the text that failed
   parsing, or where in the input it can be found. This will be improved in
   future versions.
