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

/// File create operations are logged when a file is created or overwritten. This event is useful
/// for monitoring autostart locations, like the Startup folder, as well as temporary and download
/// directories, which are common places malware drops during initial infection.
///
/// <event name="SYSMONEVENT_FILE_CREATE" value="11" level="Informational" template="File created" rulename="FileCreate" ruledefault="exclude" version="2" target="all">
///
/// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-11-filecreate>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(
    feature = "serde",
    derive(serde_crate::Serialize, serde_crate::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct FileCreateEventData<'a> {
    /// <data name="RuleName" inType="win:UnicodeString" outType="xs:string" />
    pub rule_name: Option<Cow<'a, str>>,

    /// <data name="UtcTime" inType="win:UnicodeString" outType="xs:string" />
    pub utc_time: DateTime<Utc>,

    /// <data name="ProcessGuid" inType="win:GUID" />
    pub process_guid: uuid::Uuid,

    /// <data name="ProcessId" inType="win:UInt32" outType="win:PID" />
    pub process_id: u32,

    /// <data name="Image" inType="win:UnicodeString" outType="xs:string" />
    pub image: Cow<'a, str>,

    /// <data name="TargetFilename" inType="win:UnicodeString" outType="xs:string" />
    pub target_filename: Cow<'a, str>,

    /// <data name="CreationUtcTime" inType="win:UnicodeString" outType="xs:string" />
    pub creation_utc_time: DateTime<Utc>,

    /// <data name="User" inType="win:UnicodeString" outType="xs:string" />
    pub user: Option<Cow<'a, str>>,
}

impl<'a> FileCreateEventData<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut rule_name = None;
        let mut utc_time = None;
        let mut process_guid = None;
        let mut process_id = None;
        let mut image = None;
        let mut target_filename = None;
        let mut creation_utc_time = None;
        let mut user = None;

        while let Some(token) = tokenizer.next() {
            match token? {
                Token::ElementStart { local, .. } => match local.as_str() {
                    "Data" => {
                        let name = util::get_name_attribute!(tokenizer);
                        let value = util::next_text_str_span!(tokenizer);

                        match name {
                            "RuleName" => rule_name = Some(util::unescape_xml(&value)?),
                            "UtcTime" => {
                                utc_time = Some(util::parse_utc_from_str(&value, UTC_TIME_FORMAT)?)
                            }
                            "ProcessGuid" => process_guid = Some(util::parse_win_guid_str(&value)?),
                            "ProcessId" => process_id = Some(util::parse_int::<u32>(&value)?),
                            "Image" => image = Some(util::unescape_xml(&value)?),
                            "TargetFilename" => target_filename = Some(util::unescape_xml(&value)?),
                            "CreationUtcTime" => {
                                creation_utc_time =
                                    Some(util::parse_utc_from_str(&value, UTC_TIME_FORMAT)?)
                            }
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
        let target_filename = target_filename.ok_or(Error::MissingField("TargetFilename"))?;
        let creation_utc_time = creation_utc_time.ok_or(Error::MissingField("CreationUtcTime"))?;

        Ok(FileCreateEventData {
            rule_name,
            utc_time,
            process_guid,
            process_id,
            image,
            target_filename,
            creation_utc_time,
            user,
        })
    }
}

impl<'a> TryFrom<EventData<'a>> for FileCreateEventData<'a> {
    type Error = Error;

    fn try_from(event_data: EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::FileCreate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("FileCreate")),
        }
    }
}

impl<'a, 'b: 'a> TryFrom<&'b EventData<'a>> for &FileCreateEventData<'a> {
    type Error = Error;

    fn try_from(event_data: &'b EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::FileCreate(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("FileCreate")),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use xmlparser::StrSpan;

    use super::*;

    #[test]
    fn parse_file_creation_event() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let xml = r#"<EventData>
        <Data Name='RuleName'>FileCreate-Downloads</Data>
        <Data Name='UtcTime'>2019-07-24 18:05:12.673</Data>
        <Data Name='ProcessGuid'>{87E8D3BD-9DD7-5D38-0000-00107E781D00}</Data>
        <Data Name='ProcessId'>4164</Data>
        <Data Name='Image'>C:\Users\grapltest\Downloads\dropper.exe</Data>
        <Data Name='TargetFilename'>C:\Users\grapltest\Downloads\svchost.exe</Data>
        <Data Name='CreationUtcTime'>2019-07-24 18:05:12.673</Data>
        </EventData>"#;

        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let file_create_events = FileCreateEventData::try_from(&mut tokenizer)?;

        assert_eq!(
            file_create_events,
            FileCreateEventData {
                rule_name: Some(Cow::Borrowed("FileCreate-Downloads")),
                utc_time: Utc.datetime_from_str("2019-07-24 18:05:12.673", UTC_TIME_FORMAT)?,
                process_guid: util::parse_win_guid_str(&StrSpan::from(
                    "87E8D3BD-9DD7-5D38-0000-00107E781D00"
                ))?,
                process_id: 4164,
                image: Cow::Borrowed(r#"C:\Users\grapltest\Downloads\dropper.exe"#),
                target_filename: Cow::Borrowed(r#"C:\Users\grapltest\Downloads\svchost.exe"#),
                creation_utc_time: Utc
                    .datetime_from_str("2019-07-24 18:05:12.673", UTC_TIME_FORMAT)?,
                user: None
            }
        );

        Ok(())
    }
}
