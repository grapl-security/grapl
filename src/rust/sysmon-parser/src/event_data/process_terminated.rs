use std::borrow::Cow;

use chrono::{
    DateTime,
    Utc,
};
use derive_into_owned::IntoOwned;
use xmlparser::Token;

use super::{
    EventData,
    UTC_TIME_FORMAT,
};
use crate::{
    error::{
        Error,
        Result,
    },
    util,
};

/// The process terminate event reports when a process terminates. It provides the UtcTime,
/// ProcessGuid and ProcessId of the process.
///
/// <event name="SYSMONEVENT_PROCESS_TERMINATE" value="5" level="Informational" template="Process terminated" rulename="ProcessTerminate" ruledefault="include" version="3" target="all">
///
/// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-5-process-terminated>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessTerminatedEventData<'a> {
    /// <data name="RuleName" inType="win:UnicodeString" outType="xs:string" />
    pub rule_name: Option<Cow<'a, str>>,

    /// <data name="SequenceNumber" inType="win:UInt64" />
    pub sequence_number: Option<u64>,

    /// <data name="UtcTime" inType="win:UnicodeString" outType="xs:string" />
    pub utc_time: DateTime<Utc>,

    /// <data name="ProcessGuid" inType="win:GUID" />
    pub process_guid: uuid::Uuid,

    /// <data name="ProcessId" inType="win:UInt32" outType="win:PID" />
    pub process_id: u32,

    /// <data name="Image" inType="win:UnicodeString" outType="xs:string" />
    pub image: Cow<'a, str>,

    /// <data name="User" inType="win:UnicodeString" outType="xs:string" />
    pub user: Option<Cow<'a, str>>,
}

impl<'a> ProcessTerminatedEventData<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut rule_name = None;
        let mut sequence_number = None;
        let mut utc_time = None;
        let mut process_guid = None;
        let mut process_id = None;
        let mut image = None;
        let mut user = None;

        while let Some(token) = tokenizer.next() {
            match token? {
                Token::ElementStart { local, .. } => match local.as_str() {
                    "Data" => {
                        let name = util::get_name_attribute!(tokenizer);
                        let value = util::next_text_str_span!(tokenizer);

                        match name {
                            "RuleName" => rule_name = Some(util::unescape_xml(&value)?),
                            "SequenceNumber" => {
                                sequence_number = Some(util::parse_int::<u64>(&value)?)
                            }
                            "UtcTime" => {
                                utc_time = Some(util::parse_utc_from_str(&value, UTC_TIME_FORMAT)?)
                            }
                            "ProcessGuid" => process_guid = Some(util::parse_win_guid_str(&value)?),
                            "ProcessId" => process_id = Some(util::parse_int::<u32>(&value)?),
                            "Image" => image = Some(util::unescape_xml(&value)?),
                            "User" => user = Some(util::unescape_xml(&value)?),
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Token::ElementEnd {
                    end: xmlparser::ElementEnd::Close(_, name),
                    ..
                } if name.as_str() == "EventData" => break,
                _ => {}
            }
        }

        // expected fields - present in all observed schema versions
        let utc_time = utc_time.ok_or(Error::MissingField("UtcTime"))?;
        let process_guid = process_guid.ok_or(Error::MissingField("ProcessGuid"))?;
        let process_id = process_id.ok_or(Error::MissingField("ProcessId"))?;
        let image = image.ok_or(Error::MissingField("Image"))?;

        Ok(ProcessTerminatedEventData {
            rule_name,
            sequence_number,
            utc_time,
            process_guid,
            process_id,
            image,
            user,
        })
    }
}

impl<'a> TryFrom<EventData<'a>> for ProcessTerminatedEventData<'a> {
    type Error = Error;

    fn try_from(event_data: EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::ProcessTerminate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("ProcessTerminated")),
        }
    }
}

impl<'a, 'b: 'a> TryFrom<&'b EventData<'a>> for &ProcessTerminatedEventData<'a> {
    type Error = Error;

    fn try_from(event_data: &'b EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::ProcessTerminate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("ProcessTerminated")),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use xmlparser::StrSpan;

    use super::*;

    #[test]
    fn parseprocess_terminated_event() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let xml = r#"
        <EventData>
            <Data Name="RuleName">rule_name</Data>
            <Data Name="UtcTime">2022-01-04 19:52:55.688</Data>
            <Data Name="ProcessGuid">{49e2a5f6-a597-61d4-5d7a-861de5550000}</Data>
            <Data Name="ProcessId">49521</Data>
            <Data Name="Image">/usr/bin/id</Data>
            <Data Name="User">user_name</Data>
        </EventData>"#;
        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let process_termination_event = ProcessTerminatedEventData::try_from(&mut tokenizer)?;

        assert_eq!(
            process_termination_event,
            ProcessTerminatedEventData {
                rule_name: Some(Cow::Borrowed("rule_name")),
                sequence_number: None,
                utc_time: Utc.datetime_from_str("2022-01-04 19:52:55.688", UTC_TIME_FORMAT)?,
                process_guid: util::parse_win_guid_str(&StrSpan::from(
                    "{49e2a5f6-a597-61d4-5d7a-861de5550000}"
                ))?,
                process_id: 49521,
                image: Cow::Borrowed("/usr/bin/id"),
                user: Some(Cow::Borrowed("user_name")),
            }
        );

        Ok(())
    }
}
