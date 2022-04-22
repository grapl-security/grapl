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

/// This event logs when a named file stream is created, and it generates events that log the hash
/// of the contents of the file to which the stream is assigned (the unnamed stream), as well as
/// the contents of the named stream. There are malware variants that drop their executables or
/// configuration settings via browser downloads, and this event is aimed at capturing that based
/// on the browser attaching a Zone.Identifier "mark of the web" stream.
///
/// <event name="SYSMONEVENT_FILE_CREATE_STREAM_HASH" value="15" level="Informational" template="File stream created" rulename="FileCreateStreamHash" ruledefault="exclude" version="2" target="all">
///
/// https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-15-filecreatestreamhash
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileCreateStreamHashEventData<'a> {
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

    /// <data name="Hash" inType="win:UnicodeString" outType="xs:string" />
    pub hash: Cow<'a, str>,

    /// <data name="Contents" inType="win:UnicodeString" outType="xs:string" />
    pub contents: Option<Cow<'a, str>>,

    /// <data name="User" inType="win:UnicodeString" outType="xs:string" />
    pub user: Option<Cow<'a, str>>,
}

impl<'a> FileCreateStreamHashEventData<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut rule_name = None;
        let mut utc_time = None;
        let mut process_guid = None;
        let mut process_id = None;
        let mut image = None;
        let mut target_filename = None;
        let mut creation_utc_time = None;
        let mut hash = None;
        let mut contents = None;
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
                            "Hash" => hash = Some(util::unescape_xml(&value)?),
                            "Contents" => contents = Some(util::unescape_xml(&value)?),
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
        let hash = hash.ok_or(Error::MissingField("Hash"))?;

        Ok(FileCreateStreamHashEventData {
            rule_name,
            utc_time,
            process_guid,
            process_id,
            image,
            target_filename,
            creation_utc_time,
            hash,
            contents,
            user,
        })
    }
}

impl<'a> TryFrom<EventData<'a>> for FileCreateStreamHashEventData<'a> {
    type Error = Error;

    fn try_from(event_data: EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::FileCreateStreamHash(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("FileCreateStreamHash")),
        }
    }
}

impl<'a, 'b: 'a> TryFrom<&'b EventData<'a>> for &FileCreateStreamHashEventData<'a> {
    type Error = Error;

    fn try_from(event_data: &'b EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::FileCreateStreamHash(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("FileCreateStreamHash")),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use xmlparser::StrSpan;

    use super::*;

    #[test]
    fn parse_file_creation_stream_hash_event() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let xml = r#"<EventData>
        <Data Name='RuleName'>FileStream-Downloads</Data>
        <Data Name='UtcTime'>2019-07-24 17:57:02.223</Data>
        <Data Name='ProcessGuid'>{87E8D3BD-99EC-5D38-0000-00103C3A0500}</Data>
        <Data Name='ProcessId'>3460</Data>
        <Data Name='Image'>C:\Windows\system32\browser_broker.exe</Data>
        <Data Name='TargetFilename'>C:\Users\grapltest\Downloads\ChromeSetup.exe.hi3alp5.partial</Data>
        <Data Name='CreationUtcTime'>2019-07-24 17:57:01.317</Data>
        <Data Name='Hash'>MD5=EB6A90426D5004ECABC515F2DA60019A,SHA256=958850CE9C18AB8BF03D73DE75B3A4C8F8D74F27C7C2CD2B8318731C1C757326</Data>
      </EventData>"#;

        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let file_create_events = FileCreateStreamHashEventData::try_from(&mut tokenizer)?;

        assert_eq!(
            file_create_events,
            FileCreateStreamHashEventData {
                rule_name: Some(Cow::Borrowed("FileStream-Downloads")),
                utc_time: Utc.datetime_from_str("2019-07-24 17:57:02.223", UTC_TIME_FORMAT)?,
                process_guid: util::parse_win_guid_str(&StrSpan::from(
                    "87E8D3BD-99EC-5D38-0000-00103C3A0500"
                ))?,
                process_id: 3460,
                image: Cow::Borrowed(r#"C:\Windows\system32\browser_broker.exe"#),
                target_filename: Cow::Borrowed(r#"C:\Users\grapltest\Downloads\ChromeSetup.exe.hi3alp5.partial"#),
                creation_utc_time: Utc
                    .datetime_from_str("2019-07-24 17:57:01.317", UTC_TIME_FORMAT)?,
                hash: Cow::Borrowed("MD5=EB6A90426D5004ECABC515F2DA60019A,SHA256=958850CE9C18AB8BF03D73DE75B3A4C8F8D74F27C7C2CD2B8318731C1C757326"),
                contents: None,
                user: None
            }
        );

        Ok(())
    }
}
