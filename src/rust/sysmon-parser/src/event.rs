use derive_into_owned::IntoOwned;

use crate::{
    error::Result,
    event_data::{
        self,
        EventData,
    },
    system::{
        EventId,
        System,
    },
};

/// Windows Event data of parsed Sysmon events.
///
/// `<https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon>`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SysmonEvent<'a> {
    /// Defines the information that identifies the provider and how it was enabled, the event, the
    /// channel to which the event was written, and system information such as the process and
    /// thread IDs.
    ///
    /// `<https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-systempropertiestype-complextype>`
    pub system: System<'a>,

    /// Contains data specific to the event generated
    ///
    /// `<https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#events>`
    pub event_data: EventData<'a>,
}

impl<'a> SysmonEvent<'a> {
    /// Parses a Sysmon event XML.
    ///
    /// Unsupported events types will result in `event_data: EventData::Unsupported`
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
    /// let result: Result<SysmonEvent> = sysmon_parser::SysmonEvent::from_str(xml);
    /// let event = result.unwrap();
    /// assert_eq!(event.system.computer, "hostname");
    /// ```
    pub fn from_str(input: &'a str) -> Result<SysmonEvent<'a>> {
        let mut tokenizer = xmlparser::Tokenizer::from(input);

        from_tokenizer(&mut tokenizer)
    }
}

#[inline]
fn find_start_element(tokenizer: &mut xmlparser::Tokenizer) -> Result<()> {
    for token in tokenizer.by_ref() {
        match token? {
            xmlparser::Token::ElementStart { local, .. } if local.as_str() == "Event" => {
                return Ok(());
            }
            _ => {}
        }
    }

    Err(crate::error::Error::SysmonEventNotFound)
}

pub(crate) fn from_tokenizer<'a, 'b: 'a>(
    tokenizer: &mut xmlparser::Tokenizer<'b>,
) -> Result<SysmonEvent<'a>> {
    find_start_element(tokenizer)?;

    let system = System::try_from(tokenizer)?;

    let event_data = match system.event_id {
        EventId::FileCreate => {
            EventData::FileCreate(event_data::FileCreateEventData::try_from(tokenizer)?)
        }
        EventId::NetworkConnection => {
            EventData::NetworkConnect(event_data::NetworkConnectionEventData::try_from(tokenizer)?)
        }
        EventId::ProcessCreation => {
            EventData::ProcessCreate(event_data::ProcessCreateEventData::try_from(tokenizer)?)
        }
        EventId::ProcessTerminated => EventData::ProcessTerminate(
            event_data::ProcessTerminatedEventData::try_from(tokenizer)?,
        ),
        _ => EventData::Unsupported,
    };

    // Advance tokenizer to end of event
    for token in tokenizer.by_ref() {
        match token? {
            xmlparser::Token::ElementEnd {
                end: xmlparser::ElementEnd::Close(_, name),
                ..
            } if name.as_str() == "Event" => break,

            _ => {}
        }
    }

    Ok(SysmonEvent { system, event_data })
}
