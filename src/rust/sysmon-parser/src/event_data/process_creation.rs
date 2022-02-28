use std::borrow::Cow;

use chrono::{
    DateTime,
    TimeZone,
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

/// The process creation event provides extended information about a newly created process. The
/// full command line provides context on the process execution. The ProcessGUID field is a unique
/// value for this process across a domain to make event correlation easier. The hash is a full
/// hash of the file with the algorithms in the HashType field.
///
/// <event name="SYSMONEVENT_CREATE_PROCESS" value="1" level="Informational" template="Process Create" rulename="ProcessCreate" ruledefault="include" version="5" target="all">
///
/// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-1-process-creation>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
pub struct ProcessCreateEventData<'a> {
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

    /// <data name="FileVersion" inType="win:UnicodeString" outType="xs:string" />
    pub file_version: Option<Cow<'a, str>>,

    /// <data name="Description" inType="win:UnicodeString" outType="xs:string" />
    pub description: Option<Cow<'a, str>>,

    /// <data name="Product" inType="win:UnicodeString" outType="xs:string" />
    pub product: Option<Cow<'a, str>>,

    /// <data name="Company" inType="win:UnicodeString" outType="xs:string" />
    pub company: Option<Cow<'a, str>>,

    /// <data name="OriginalFileName" inType="win:UnicodeString" outType="xs:string" />
    pub original_file_name: Option<Cow<'a, str>>,

    /// <data name="CommandLine" inType="win:UnicodeString" outType="xs:string" />
    pub command_line: Cow<'a, str>,

    /// <data name="CurrentDirectory" inType="win:UnicodeString" outType="xs:string" />
    pub current_directory: Cow<'a, str>,

    /// <data name="User" inType="win:UnicodeString" outType="xs:string" />
    pub user: Cow<'a, str>,

    /// <data name="LogonGuid" inType="win:GUID" />
    pub logon_guid: uuid::Uuid,

    /// <data name="LogonId" inType="win:HexInt64" />
    pub logon_id: u64,

    /// <data name="TerminalSessionId" inType="win:UInt32" />
    pub terminal_session_id: u32,

    /// <data name="IntegrityLevel" inType="win:UnicodeString" outType="xs:string" />
    pub integrity_level: Cow<'a, str>,

    /// <data name="Hashes" inType="win:UnicodeString" outType="xs:string" />
    pub hashes: Cow<'a, str>,

    /// <data name="ParentProcessGuid" inType="win:GUID" />
    pub parent_process_guid: uuid::Uuid,

    /// <data name="ParentProcessId" inType="win:UInt32" outType="win:PID" />
    pub parent_process_id: u32,

    /// <data name="ParentImage" inType="win:UnicodeString" outType="xs:string" />
    pub parent_image: Cow<'a, str>,

    /// <data name="ParentCommandLine" inType="win:UnicodeString" outType="xs:string" />
    pub parent_command_line: Cow<'a, str>,

    /// <data name="ParentUser" inType="win:UnicodeString" outType="xs:string" />
    pub parent_user: Option<Cow<'a, str>>,
}

impl<'a> ProcessCreateEventData<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut rule_name = None;
        let mut utc_time = None;
        let mut process_guid = None;
        let mut process_id = None;
        let mut image = None;
        let mut file_version = None;
        let mut description = None;
        let mut product = None;
        let mut company = None;
        let mut original_file_name = None;
        let mut command_line = None;
        let mut current_directory = None;
        let mut user = None;
        let mut logon_guid = None;
        let mut logon_id = None;
        let mut terminal_session_id = None;
        let mut integrity_level = None;
        let mut hashes = None;
        let mut parent_process_guid = None;
        let mut parent_process_id = None;
        let mut parent_image = None;
        let mut parent_command_line = None;
        let mut parent_user = None;
        let mut sequence_number = None;

        while let Some(token) = tokenizer.next() {
            match token? {
                Token::ElementStart { local, .. } => match local.as_str() {
                    "Data" => {
                        let name = util::get_name_attribute!(tokenizer);
                        let text = util::next_text_str!(tokenizer);
                        match name {
                            "RuleName" => rule_name = Some(util::unescape_xml(text)?),
                            "SequenceNumber" => sequence_number = Some(text.parse::<u64>()?),
                            "UtcTime" => {
                                utc_time = Some(Utc.datetime_from_str(text, UTC_TIME_FORMAT))
                            }
                            "ProcessGuid" => process_guid = Some(util::parse_win_guid_str(text)),
                            "ProcessId" => process_id = Some(text.parse::<u32>()),
                            "Image" => image = Some(util::unescape_xml(text)),
                            "FileVersion" => file_version = Some(util::unescape_xml(text)?),
                            "Description" => description = Some(util::unescape_xml(text)?),
                            "Product" => product = Some(util::unescape_xml(text)?),
                            "Company" => company = Some(util::unescape_xml(text)?),
                            "OriginalFileName" => {
                                original_file_name = Some(util::unescape_xml(text)?)
                            }
                            "CommandLine" => command_line = Some(util::unescape_xml(text)),
                            "CurrentDirectory" => {
                                current_directory = Some(util::unescape_xml(text))
                            }
                            "User" => user = Some(util::unescape_xml(text)),
                            "LogonGuid" => logon_guid = Some(util::parse_win_guid_str(text)),
                            "LogonId" => logon_id = Some(util::from_zero_or_hex_str(text)),
                            "TerminalSessionId" => terminal_session_id = Some(text.parse::<u32>()),
                            "IntegrityLevel" => integrity_level = Some(util::unescape_xml(text)),
                            "Hashes" => hashes = Some(util::unescape_xml(text)),
                            "ParentProcessGuid" => {
                                parent_process_guid = Some(util::parse_win_guid_str(text))
                            }
                            "ParentProcessId" => parent_process_id = Some(text.parse::<u32>()),
                            "ParentImage" => parent_image = Some(util::unescape_xml(text)),
                            "ParentCommandLine" => {
                                parent_command_line = Some(util::unescape_xml(text))
                            }
                            "ParentUser" => parent_user = Some(util::unescape_xml(text)?),
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
        let utc_time = utc_time.ok_or(Error::MissingField("UtcTime"))??;
        let process_guid = process_guid.ok_or(Error::MissingField("ProcessGuid"))??;
        let process_id = process_id.ok_or(Error::MissingField("ProcessId"))??;
        let image = image.ok_or(Error::MissingField("Image"))??;
        let command_line = command_line.ok_or(Error::MissingField("CommandLine"))??;
        let current_directory =
            current_directory.ok_or(Error::MissingField("CurrentDirectory"))??;
        let user = user.ok_or(Error::MissingField("User"))??;
        let logon_guid = logon_guid.ok_or(Error::MissingField("LogonGuid"))??;
        let logon_id = logon_id.ok_or(Error::MissingField("LogonId"))??;
        let terminal_session_id =
            terminal_session_id.ok_or(Error::MissingField("TerminalSessionId"))??;
        let integrity_level = integrity_level.ok_or(Error::MissingField("IntegrityLevel"))??;
        let hashes = hashes.ok_or(Error::MissingField("Hashes"))??;
        let parent_process_guid =
            parent_process_guid.ok_or(Error::MissingField("ParentProcessGuid"))??;
        let parent_process_id =
            parent_process_id.ok_or(Error::MissingField("ParentProcessId"))??;
        let parent_image = parent_image.ok_or(Error::MissingField("ParentImage"))??;
        let parent_command_line =
            parent_command_line.ok_or(Error::MissingField("ParentCommandLine"))??;

        Ok(ProcessCreateEventData {
            rule_name,
            utc_time,
            process_guid,
            process_id,
            image,
            file_version,
            description,
            product,
            company,
            original_file_name,
            command_line,
            current_directory,
            user,
            logon_guid,
            logon_id,
            terminal_session_id,
            integrity_level,
            hashes,
            parent_process_guid,
            parent_process_id,
            parent_image,
            parent_command_line,
            parent_user,
            sequence_number,
        })
    }
}

impl<'a> TryFrom<EventData<'a>> for ProcessCreateEventData<'a> {
    type Error = Error;

    fn try_from(event_data: EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::ProcessCreate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("ProcessCreate")),
        }
    }
}

impl<'a, 'b: 'a> TryFrom<&'b EventData<'a>> for &ProcessCreateEventData<'a> {
    type Error = Error;

    fn try_from(event_data: &'b EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::ProcessCreate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("ProcessCreate")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_process_creation_event() -> Result<()> {
        let xml = r#"
        <EventData>
            <Data Name="RuleName">rule_name</Data>
            <Data Name="UtcTime">2022-01-04 19:54:15.661</Data>
            <Data Name="ProcessGuid">{49e2a5f6-a5e7-61d4-119e-dc77a5550000}</Data>
            <Data Name="ProcessId">49570</Data>
            <Data Name="Image">/usr/bin/tr</Data>
            <Data Name="FileVersion">-</Data>
            <Data Name="Description">-</Data>
            <Data Name="Product">-</Data>
            <Data Name="Company">-</Data>
            <Data Name="OriginalFileName">-</Data>
            <Data Name="CommandLine">tr [:upper:][:lower:]</Data>
            <Data Name="CurrentDirectory">/root</Data>
            <Data Name="User">root</Data>
            <Data Name="LogonGuid">{49e2a5f6-0000-0000-0000-000000000000}</Data>
            <Data Name="LogonId">0</Data>
            <Data Name="TerminalSessionId">3</Data>
            <Data Name="IntegrityLevel">no level</Data>
            <Data Name="Hashes">-</Data>
            <Data Name="ParentProcessGuid">{00000000-0000-0000-0000-000000000000}</Data>
            <Data Name="ParentProcessId">49568</Data>
            <Data Name="ParentImage">-</Data>
            <Data Name="ParentCommandLine">-</Data>
            <Data Name="ParentUser">-</Data>
        </EventData>"#;

        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let process_creation_event = ProcessCreateEventData::try_from(&mut tokenizer)?;

        assert_eq!(
            process_creation_event,
            ProcessCreateEventData {
                rule_name: Some(Cow::Borrowed("rule_name")),
                utc_time: Utc.datetime_from_str("2022-01-04 19:54:15.661", UTC_TIME_FORMAT)?,
                process_guid: util::parse_win_guid_str("49e2a5f6-a5e7-61d4-119e-dc77a5550000")?,
                process_id: 49570,
                image: Cow::Borrowed("/usr/bin/tr"),
                file_version: Some(Cow::Borrowed("-")),
                description: Some(Cow::Borrowed("-")),
                product: Some(Cow::Borrowed("-")),
                company: Some(Cow::Borrowed("-")),
                original_file_name: Some(Cow::Borrowed("-")),
                command_line: Cow::Borrowed("tr [:upper:][:lower:]"),
                current_directory: Cow::Borrowed("/root"),
                user: Cow::Borrowed("root"),
                logon_guid: util::parse_win_guid_str("49e2a5f6-0000-0000-0000-000000000000")?,
                logon_id: 0,
                terminal_session_id: 3,
                integrity_level: Cow::Borrowed("no level"),
                hashes: Cow::Borrowed("-"),
                parent_process_guid: util::parse_win_guid_str(
                    "00000000-0000-0000-0000-000000000000"
                )?,
                parent_process_id: 49568,
                parent_image: Cow::Borrowed("-"),
                parent_command_line: Cow::Borrowed("-"),
                parent_user: Some(Cow::Borrowed("-")),
                sequence_number: None,
            }
        );

        Ok(())
    }
}
