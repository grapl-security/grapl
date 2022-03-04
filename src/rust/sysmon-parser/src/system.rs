use std::{
    borrow::Cow,
    str::FromStr,
};

use chrono::{
    DateTime,
    Utc,
};
use derive_into_owned::IntoOwned;
use xmlparser::Token;

use crate::{
    error::{
        Error,
        Result,
    },
    util,
};

mod correlation;
mod event_id;
mod execution;
mod provider;
mod security;
mod time_created;

pub use correlation::Correlation;
pub use event_id::EventId;
pub use execution::Execution;
pub use provider::Provider;
pub use security::Security;
pub use time_created::TimeCreated;

/// Defines the information that identifies the provider and how it was enabled, the event, the
/// channel to which the event was written, and system information such as the process and thread
/// IDs.
///
/// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-systempropertiestype-complextype>
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
pub struct System<'a> {
    /// Identifies the provider that logged the event.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-provider-systempropertiestype-element>
    pub provider: provider::Provider<'a>,

    /// The identifier that the provider used to identify the event.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-eventid-systempropertiestype-element>
    pub event_id: event_id::EventId,

    /// The version number of the event's definition.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/schema-version-systempropertiestype-element>
    pub version: u8,

    /// The severity level defined in the event.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-level-systempropertiestype-element>
    pub level: u8,

    /// The task defined in the event. Task and opcode are typically used to identify the location
    /// in the application from where the event was logged.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-task-systempropertiestype-element>
    pub task: u16,

    /// The opcode defined in the event. Task and opcode are typcially used to identify the
    /// location in the application from where the event was logged.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-opcode-systempropertiestype-element>
    pub opcode: u8,

    /// A bitmask of the keywords defined in the event. Keywords are used to classify types of
    /// events (for example, events associated with reading data).
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-keywords-systempropertiestype-element>
    pub keywords: u64,

    /// The time stamp that identifies when the event was logged. The time stamp will include
    /// either the SystemTime attribute or the RawTime attribute.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-timecreated-systempropertiestype-element>
    pub time_created: time_created::TimeCreated,

    /// The record number assigned to the event when it was logged.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-eventrecordid-systempropertiestype-element>
    pub event_record_id: u64,

    /// The activity identifiers that consumers can use to group related events together.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-correlation-systempropertiestype-element>
    pub correlation: correlation::Correlation,

    /// Contains information about the process and thread that logged the event.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-execution-systempropertiestype-element>
    pub execution: execution::Execution,

    /// The channel to which the event was logged.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-channel-systempropertiestype-element>
    pub channel: Cow<'a, str>,

    /// The name of the computer on which the event occurred.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-computer-systempropertiestype-element>
    pub computer: Cow<'a, str>,

    /// Identifies the user that logged the event.
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/wes/eventschema-security-systempropertiestype-element>
    pub security: security::Security<'a>,
}

impl<'a> System<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut provider = None;
        let mut event_id = None;
        let mut version = None;
        let mut level = None;
        let mut task = None;
        let mut opcode = None;
        let mut keywords = None;
        let mut time_created = None;
        let mut event_record_id = None;
        let mut correlation = None;
        let mut execution = None;
        let mut channel = None;
        let mut computer = None;
        let mut security = None;

        while let Some(token) = tokenizer.next() {
            match token? {
                Token::ElementStart { local, .. } => {
                    match local.as_str() {
                        "Provider" => {
                            let mut name: Option<Cow<str>> = None;
                            let mut guid: Option<uuid::Uuid> = None;
                            let mut event_source_name: Option<Cow<str>> = None;

                            for token in tokenizer.by_ref() {
                                match token? {
                                    Token::Attribute { local, value, .. } => match local.as_str() {
                                        "Name" => name = Some(util::unescape_xml(&value)?),
                                        "Guid" => guid = Some(util::parse_win_guid_str(&value)?),
                                        "EventSourceName" => {
                                            event_source_name = Some(util::unescape_xml(&value)?)
                                        }
                                        _ => {}
                                    },
                                    Token::ElementEnd { .. } => break,
                                    _ => {}
                                }
                            }

                            provider = Some(provider::Provider {
                                name,
                                guid,
                                event_source_name,
                            })
                        }
                        "EventID" => {
                            let value = util::next_text_str_span!(tokenizer);
                            event_id = Some(event_id::EventId::from_str(value.as_str())?);
                        }
                        "Version" => {
                            let value = util::next_text_str_span!(tokenizer);
                            version = Some(util::parse_int::<u8>(&value)?);
                        }
                        "Level" => {
                            let value = util::next_text_str_span!(tokenizer);
                            level = Some(util::parse_int::<u8>(&value)?);
                        }
                        "Task" => {
                            let value = util::next_text_str_span!(tokenizer);
                            task = Some(util::parse_int::<u16>(&value)?);
                        }
                        "Opcode" => {
                            let value = util::next_text_str_span!(tokenizer);
                            opcode = Some(util::parse_int::<u8>(&value)?);
                        }
                        "Keywords" => {
                            let value = util::next_text_str_span!(tokenizer);
                            keywords = Some(util::from_zero_or_hex_str(&value)?);
                        }
                        "TimeCreated" => {
                            let mut system_time: Option<DateTime<Utc>> = None;

                            for token in tokenizer.by_ref() {
                                match token? {
                                    Token::Attribute { local, value, .. } => match local.as_str() {
                                        "SystemTime" => {
                                            system_time = Some(util::parse_utc(&value)?)
                                        }
                                        _ => {}
                                    },
                                    Token::ElementEnd { .. } => break,
                                    _ => {}
                                }
                            }

                            let system_time =
                                system_time.ok_or(Error::MissingField("system_time"))?;

                            time_created = Some(time_created::TimeCreated { system_time })
                        }
                        "EventRecordID" => {
                            let value = util::next_text_str_span!(tokenizer);

                            event_record_id = Some(util::parse_int::<u64>(&value)?);
                        }
                        "Correlation" => {
                            let mut activity_id: Option<uuid::Uuid> = None;
                            let mut related_activity_id: Option<uuid::Uuid> = None;

                            for token in tokenizer.by_ref() {
                                match token? {
                                    Token::Attribute { local, value, .. } => match local.as_str() {
                                        "ActivityID" => {
                                            activity_id = Some(util::parse_win_guid_str(&value)?)
                                        }
                                        "RelatedActivityID" => {
                                            related_activity_id =
                                                Some(util::parse_win_guid_str(&value)?)
                                        }
                                        _ => {}
                                    },
                                    Token::ElementEnd { .. } => break,
                                    _ => {}
                                }
                            }

                            correlation = Some(correlation::Correlation {
                                activity_id,
                                related_activity_id,
                            })
                        }
                        "Execution" => {
                            let mut process_id: Option<u32> = None;
                            let mut thread_id: Option<u32> = None;

                            // optional fields
                            let mut processor_id: Option<u8> = None;
                            let mut session_id: Option<u32> = None;
                            let mut kernel_time: Option<u32> = None;
                            let mut user_time: Option<u32> = None;
                            let mut processor_time: Option<u32> = None;

                            for token in tokenizer.by_ref() {
                                match token? {
                                    Token::Attribute { local, value, .. } => match local.as_str() {
                                        "ProcessID" => {
                                            process_id = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        "ThreadID" => {
                                            thread_id = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        "ProcessorID" => {
                                            processor_id = Some(util::parse_int::<u8>(&value)?);
                                        }
                                        "SessionID" => {
                                            session_id = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        "KernelTime" => {
                                            kernel_time = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        "UserTime" => {
                                            user_time = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        "ProcessorTime" => {
                                            processor_time = Some(util::parse_int::<u32>(&value)?);
                                        }
                                        _ => {}
                                    },
                                    Token::ElementEnd { .. } => break,
                                    _ => {}
                                }
                            }

                            let process_id = process_id.ok_or(Error::MissingField("process_id"))?;
                            let thread_id = thread_id.ok_or(Error::MissingField("thread_id"))?;

                            execution = Some(execution::Execution {
                                process_id,
                                thread_id,
                                processor_id,
                                session_id,
                                kernel_time,
                                user_time,
                                processor_time,
                            })
                        }
                        "Channel" => {
                            let value = util::next_text_str_span!(tokenizer);
                            channel = Some(util::unescape_xml(&value)?);
                        }
                        "Computer" => {
                            let value = util::next_text_str_span!(tokenizer);
                            computer = Some(util::unescape_xml(&value)?);
                        }
                        "Security" => {
                            let mut user_id: Option<Cow<str>> = None;

                            for token in tokenizer.by_ref() {
                                match token? {
                                    Token::Attribute { local, value, .. } => match local.as_str() {
                                        "UserId" => {
                                            user_id = Some(util::unescape_xml(&value)?);
                                        }
                                        _ => {}
                                    },
                                    Token::ElementEnd { .. } => break,
                                    _ => {}
                                }
                            }

                            security = Some(security::Security { user_id })
                        }
                        // skip unknown
                        _ => {}
                    }
                }
                Token::ElementEnd {
                    end: xmlparser::ElementEnd::Close(_, name),
                    ..
                } if name.as_str() == "System" => break,
                _ => {}
            }
        }

        let provider = provider.ok_or(Error::MissingField("provider"))?;
        let event_id = event_id.ok_or(Error::MissingField("event_id"))?;
        let version = version.ok_or(Error::MissingField("version"))?;
        let level = level.ok_or(Error::MissingField("level"))?;
        let task = task.ok_or(Error::MissingField("task"))?;
        let opcode = opcode.ok_or(Error::MissingField("opcode"))?;
        let keywords = keywords.ok_or(Error::MissingField("keywords"))?;
        let time_created = time_created.ok_or(Error::MissingField("time_created"))?;
        let event_record_id = event_record_id.ok_or(Error::MissingField("event_record_id"))?;
        let correlation = correlation.ok_or(Error::MissingField("correlation"))?;
        let execution = execution.ok_or(Error::MissingField("execution"))?;
        let channel = channel.ok_or(Error::MissingField("channel"))?;
        let computer = computer.ok_or(Error::MissingField("computer"))?;
        let security = security.ok_or(Error::MissingField("security"))?;

        Ok(System {
            provider,
            event_id,
            version,
            level,
            task,
            opcode,
            keywords,
            time_created,
            event_record_id,
            correlation,
            execution,
            channel,
            computer,
            security,
        })
    }
}

#[cfg(test)]
mod tests {
    use xmlparser::StrSpan;

    use super::*;

    #[test]
    fn parse_event_system() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let xml = r#"
    <System>
        <Provider Name="Linux-Sysmon" Guid="{ff032593-a8d3-4f13-b0d6-01fc615a0f97}"/>
        <EventID>1</EventID>
        <Version>5</Version>
        <Level>4</Level>
        <Task>1</Task>
        <Opcode>0</Opcode>
        <Keywords>0x8000000000000000</Keywords>
        <TimeCreated SystemTime="2022-01-04T19:54:15.665400000Z"/>
        <EventRecordID>77</EventRecordID>
        <Correlation/>
        <Execution ProcessID="49514" ThreadID="49514"/>
        <Channel>Linux-Sysmon/Operational</Channel>
        <Computer>tux</Computer>
        <Security UserId="0"/>
    </System>"#;

        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let system = System::try_from(&mut tokenizer)?;

        assert_eq!(
            system,
            System {
                provider: provider::Provider {
                    name: Some(Cow::Borrowed("Linux-Sysmon")),
                    guid: Some(util::parse_win_guid_str(&StrSpan::from(
                        "ff032593-a8d3-4f13-b0d6-01fc615a0f97"
                    ))?),
                    event_source_name: None
                },
                event_id: event_id::EventId::ProcessCreation,
                version: 5,
                level: 4,
                task: 1,
                opcode: 0,
                keywords: 0x8000000000000000,
                time_created: time_created::TimeCreated {
                    system_time: "2022-01-04T19:54:15.665400000Z".parse::<DateTime<Utc>>()?
                },
                event_record_id: 77,
                correlation: correlation::Correlation {
                    activity_id: None,
                    related_activity_id: None
                },
                execution: execution::Execution {
                    process_id: 49514,
                    thread_id: 49514,
                    processor_id: None,
                    session_id: None,
                    kernel_time: None,
                    user_time: None,
                    processor_time: None
                },
                channel: Cow::Borrowed(r#"Linux-Sysmon/Operational"#),
                computer: Cow::Borrowed("tux"),
                security: security::Security {
                    user_id: Some(Cow::Borrowed("0"))
                },
            }
        );

        Ok(())
    }
}
