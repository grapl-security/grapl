extern crate anyhow;
extern crate chrono;
#[macro_use]
extern crate derive_is_enum_variant;
extern crate serde;
extern crate serde_xml_rs;
extern crate uuid;

use std::{collections::HashMap, convert::TryFrom, str::FromStr};

use anyhow::{anyhow, Result};
use chrono::prelude::*;
use failure::_core::ops::Deref;
use serde::{de::Error as SerdeError, Deserialize, Deserializer};

macro_rules! get_or_err {
    ($map:ident, $field_name:expr) => {
        match $map.remove($field_name) {
            Some(field) => field,
            None => return Err(anyhow!("No field: {}", $field_name)),
        }
    };

    ($map:ident, $field_name:expr, $maperr:expr) => {
        match $map.remove($field_name) {
            Some(field) => field,
            None => return Err(anyhow!("No field: {}", $field_name)).map_err($maperr),
        }
    };
}

#[derive(Debug, Clone, Hash, is_enum_variant)]
pub enum Event {
    ProcessCreate(ProcessCreateEvent),
    FileCreate(FileCreateEvent),
    InboundNetwork(NetworkEvent),
    OutboundNetwork(NetworkEvent),
}

impl FromStr for Event {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_xml_rs::from_str::<ProcessCreateEvent>(s)
            .map(|p| Event::ProcessCreate(p))
            .or_else(|_| serde_xml_rs::from_str::<FileCreateEvent>(s).map(|f| Event::FileCreate(f)))
            .or_else(|_| {
                serde_xml_rs::from_str::<NetworkEvent>(s).map(|n| {
                    if n.event_data.initiated {
                        Event::OutboundNetwork(n)
                    } else {
                        Event::InboundNetwork(n)
                    }
                })
            })
            .map_err(|e| anyhow!("Error : {:?} {}", e, s))
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Provider {
    #[serde(rename = "Name")]
    pub provider_name: String,
    #[serde(rename = "Guid")]
    pub provider_guid: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct EventId {
    #[serde(rename = "$value")]
    pub event_id: u8,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Level {
    #[serde(rename = "$value")]
    pub level: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Task {
    #[serde(rename = "$value")]
    pub task: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Version {
    #[serde(rename = "$value")]
    pub version: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Opcode {
    #[serde(rename = "$value")]
    pub opcode: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Keywords {
    #[serde(rename = "$value")]
    pub keywords: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct TimeCreated {
    #[serde(rename = "SystemTime")]
    pub system_time: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct EventRecordId {
    #[serde(rename = "$value")]
    pub event_record_id: u32,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Execution {
    #[serde(rename = "ProcessID")]
    pub process_id: String,
    #[serde(rename = "ThreadID")]
    pub thread_id: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Channel {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Computer {
    #[serde(rename = "$value")]
    pub computer: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Security {
    #[serde(rename = "UserID")]
    pub security: String,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct System {
    /// <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}" />
    #[serde(rename = "Provider")]
    pub provider: Provider,
    /// <EventID>1</EventID>
    #[serde(rename = "EventID")]
    pub event_id: EventId,
    /// <Version>5</Version>
    #[serde(rename = "Version")]
    pub version: Version,
    /// <Level>4</Level>
    #[serde(rename = "Level")]
    pub level: Level,
    /// <Task>1</Task>
    #[serde(rename = "Task")]
    pub task: Task,
    /// <Opcode>0</Opcode>
    #[serde(rename = "Opcode")]
    pub opcode: Opcode,
    /// <Keywords>0x8000000000000000</Keywords>
    #[serde(rename = "Keywords")]
    pub keywords: Keywords,
    /// <TimeCreated SystemTime="2017-04-28T22:08:22.025812200Z" />
    #[serde(rename = "TimeCreated")]
    pub time_created: TimeCreated,
    /// <EventRecordID>9947</EventRecordID>
    #[serde(rename = "EventRecordID")]
    pub event_record_id: EventRecordId,
    /// <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
    /// <Execution ProcessID="3216" ThreadID="3964" />
    #[serde(rename = "Execution")]
    pub execution: Execution,
    /// <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
    #[serde(rename = "Channel")]
    pub channel: Channel,
    /// <Computer>rfsH.lab.local</Computer>
    #[serde(rename = "Computer")]
    pub computer: Computer,
    /// <Security UserID="S-1-5-18" />
    #[serde(rename = "Security")]
    pub security: Security,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct UtcTime {
    #[serde(rename = "$value")]
    pub utc_time: String,
}

impl Deref for UtcTime {
    type Target = str;

    fn deref(&self) -> &str {
        &self.utc_time
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct ProcessGuid {
    pub process_guid: uuid::Uuid,
}

impl ProcessGuid {
    pub fn get_creation_timestamp(&self) -> u64 {
        let guid = self.process_guid.as_bytes();

        const OFF: usize = 4;

        // big endian :|
        let b = [guid[OFF + 1], guid[OFF], guid[OFF + 3], guid[OFF + 2]];

        let ts = i32::from_le_bytes(b);
        Utc.timestamp(ts as i64, 0).timestamp() as u64
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Image {
    pub image: String,
}

impl Deref for Image {
    type Target = str;

    fn deref(&self) -> &str {
        &self.image
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct CommandLine {
    pub command_line: String,
}

impl Deref for CommandLine {
    type Target = str;

    fn deref(&self) -> &str {
        &self.command_line
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct CurrentDirectory {
    pub current_directory: String,
}

impl Deref for CurrentDirectory {
    type Target = str;

    fn deref(&self) -> &str {
        &self.current_directory
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct User {
    pub user: String,
}

impl Deref for User {
    type Target = str;

    fn deref(&self) -> &str {
        &self.user
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct LogonGuid {
    pub logon_guid: uuid::Uuid,
}

impl Deref for LogonGuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.logon_guid
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct LogonId {
    pub logon_id: String,
}

impl Deref for LogonId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.logon_id
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct TerminalSessionId {
    pub terminal_session_id: String,
}

impl Deref for TerminalSessionId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.terminal_session_id
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct IntegrityLevel {
    pub integrity_level: String,
}

impl Deref for IntegrityLevel {
    type Target = str;

    fn deref(&self) -> &str {
        &self.integrity_level
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Hashes {
    pub hashes: String,
}

impl Deref for Hashes {
    type Target = str;

    fn deref(&self) -> &str {
        &self.hashes
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct TargetFilename {
    pub target_filename: String,
}

impl Deref for TargetFilename {
    type Target = str;

    fn deref(&self) -> &str {
        &self.target_filename
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct ProcessCreateEventData {
    /// <Data Name="UtcTime">2017-04-28 22:08:22.025</Data>
    pub utc_time: UtcTime,
    /// <Data Name="ProcessGuid">{A23EAE89-BD56-5903-0000-0010E9D95E00}</Data>
    pub process_guid: ProcessGuid,
    /// <Data Name="ProcessId">6228</Data>
    pub process_id: u64,
    /// <Data Name="Image">C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
    pub image: Image,
    /// <Data Name="CommandLine">"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" --type=utility --lang=en-US --no-sandbox --service-request-channel-token=F47498BBA884E523FA93E623C4569B94 --mojo-platform-channel-handle=3432 /prefetch:8</Data>
    pub command_line: CommandLine,
    /// <Data Name="CurrentDirectory">C:\Program Files (x86)\Google\Chrome\Application\58.0.3029.81\</Data>
    pub current_directory: CurrentDirectory,
    /// <Data Name="User">LAB\rsmith</Data>
    pub user: User,
    /// <Data Name="LogonGuid">{A23EAE89-B357-5903-0000-002005EB0700}</Data>
    pub logon_guid: LogonGuid,
    /// <Data Name="LogonId">0x7eb05</Data>
    pub logon_id: LogonId,
    /// <Data Name="TerminalSessionId">1</Data>
    pub terminal_session_id: TerminalSessionId,
    /// <Data Name="IntegrityLevel">Medium</Data>
    pub integrity_level: IntegrityLevel,
    /// <Data Name="Hashes">SHA256=6055A20CF7EC81843310AD37700FF67B2CF8CDE3DCE68D54BA42934177C10B57</Data>
    pub hashes: Hashes,
    /// <Data Name="ParentProcessGuid">{A23EAE89-BD28-5903-0000-00102F345D00}</Data>
    pub parent_process_guid: ProcessGuid,
    /// <Data Name="ParentProcessId">13220</Data>
    pub parent_process_id: u64,
    /// <Data Name="ParentImage">C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
    pub parent_image: Image,
    /// <Data Name="ParentCommandLine">"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" </Data>
    pub parent_command_line: CommandLine,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct ProcessCreateEvent {
    #[serde(rename = "System")]
    pub system: System,
    #[serde(rename = "EventData", deserialize_with = "from_intermediary_data")]
    pub event_data: ProcessCreateEventData,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct FileCreateEventData {
    pub utc_time: UtcTime,
    pub process_guid: ProcessGuid,
    pub process_id: u64,
    pub image: Image,
    pub target_filename: String,
    pub creation_utc_time: UtcTime,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct FileCreateEvent {
    #[serde(rename = "System")]
    pub system: System,

    #[serde(rename = "EventData", deserialize_with = "from_intermediary_data")]
    pub event_data: FileCreateEventData,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct NetworkEventData {
    pub utc_time: UtcTime,
    pub process_guid: ProcessGuid,
    pub process_id: u64,
    pub image: Image,
    pub user: Option<User>,
    pub protocol: String,
    pub initiated: bool,
    pub source_is_ipv6: String,
    pub source_ip: String,
    pub source_hostname: Option<String>,
    pub source_port: u16,
    pub source_port_name: Option<String>,
    pub destination_is_ipv6: String,
    pub destination_ip: String,
    pub destination_hostname: Option<String>,
    pub destination_port: u16,
    pub destination_port_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct NetworkEvent {
    #[serde(rename = "System")]
    pub system: System,
    #[serde(rename = "EventData", deserialize_with = "from_intermediary_data")]
    pub event_data: NetworkEventData,
}

impl TryFrom<IntermediaryEventData> for ProcessCreateEventData {
    type Error = anyhow::Error;

    fn try_from(inter: IntermediaryEventData) -> Result<Self> {
        let mut m = HashMap::with_capacity(inter.data.len());

        for data in inter.data {
            if let Some(value) = data.value {
                //                if (value == "4") {
                //                    panic!("{} {}", data.name, value.len());
                //                }
                m.insert(data.name, value);
            }
        }

        let process_id = get_or_err!(m, "ProcessId");
        let process_id: u64 = process_id.parse()?;

        let parent_process_id = get_or_err!(m, "ParentProcessId");
        let parent_process_id: u64 = parent_process_id.parse()?;

        Ok(ProcessCreateEventData {
            utc_time: UtcTime {
                utc_time: get_or_err!(m, "UtcTime"),
            },
            process_guid: ProcessGuid {
                process_guid: uuid::Uuid::parse_str(&get_or_err!(m, "ProcessGuid")[1..37])?,
            },
            process_id,
            image: Image {
                image: get_or_err!(m, "Image"),
            },
            command_line: CommandLine {
                command_line: get_or_err!(m, "CommandLine"),
            },
            current_directory: CurrentDirectory {
                current_directory: get_or_err!(m, "CurrentDirectory"),
            },
            user: User {
                user: get_or_err!(m, "User"),
            },
            logon_guid: LogonGuid {
                logon_guid: uuid::Uuid::parse_str(&get_or_err!(m, "LogonGuid")[1..37])?,
            },
            logon_id: LogonId {
                logon_id: get_or_err!(m, "LogonId"),
            },
            terminal_session_id: TerminalSessionId {
                terminal_session_id: get_or_err!(m, "TerminalSessionId"),
            },
            integrity_level: IntegrityLevel {
                integrity_level: get_or_err!(m, "IntegrityLevel"),
            },
            hashes: Hashes {
                hashes: get_or_err!(m, "Hashes"),
            },
            parent_process_guid: ProcessGuid {
                process_guid: uuid::Uuid::parse_str(&get_or_err!(m, "ParentProcessGuid")[1..37])?,
            },
            parent_process_id,
            parent_image: Image {
                image: get_or_err!(m, "ParentImage"),
            },
            parent_command_line: CommandLine {
                command_line: get_or_err!(m, "ParentCommandLine"),
            },
        })
    }
}

impl TryFrom<IntermediaryEventData> for FileCreateEventData {
    type Error = anyhow::Error;

    fn try_from(inter: IntermediaryEventData) -> Result<Self> {
        let mut m = HashMap::with_capacity(inter.data.len());

        for data in inter.data {
            if let Some(value) = data.value {
                m.insert(data.name, value);
            }
        }

        let process_id = get_or_err!(m, "ProcessId");
        let process_id = process_id.parse()?;

        Ok(FileCreateEventData {
            utc_time: UtcTime {
                utc_time: get_or_err!(m, "UtcTime"),
            },
            process_guid: ProcessGuid {
                process_guid: uuid::Uuid::parse_str(&get_or_err!(m, "ProcessGuid")[1..37])?,
            },
            process_id,
            image: Image {
                image: get_or_err!(m, "Image"),
            },
            creation_utc_time: UtcTime {
                utc_time: get_or_err!(m, "CreationUtcTime"),
            },
            target_filename: get_or_err!(m, "TargetFilename"),
        })
    }
}

impl TryFrom<IntermediaryEventData> for NetworkEventData {
    type Error = anyhow::Error;

    fn try_from(inter: IntermediaryEventData) -> Result<Self> {
        let mut m = HashMap::with_capacity(inter.data.len());

        for data in inter.data {
            if let Some(value) = data.value {
                m.insert(data.name, value);
            }
        }

        let user = m.remove("User").map(|user| User { user });

        Ok(NetworkEventData {
            utc_time: UtcTime {
                utc_time: get_or_err!(m, "UtcTime"),
            },
            process_guid: ProcessGuid {
                process_guid: uuid::Uuid::parse_str(&get_or_err!(m, "ProcessGuid")[1..37])?,
            },
            process_id: get_or_err!(m, "ProcessId").parse()?,
            image: Image {
                image: get_or_err!(m, "Image"),
            },
            user,
            protocol: get_or_err!(m, "Protocol"),
            source_is_ipv6: get_or_err!(m, "SourceIsIpv6"),
            source_ip: get_or_err!(m, "SourceIp"),
            source_hostname: m.remove("SourceHostname"),
            source_port_name: m.remove("SourcePortName"),
            destination_is_ipv6: get_or_err!(m, "DestinationIsIpv6"),
            destination_ip: get_or_err!(m, "DestinationIp"),
            destination_hostname: m.remove("DestinationHostname"),
            destination_port_name: m.remove("DestinationPortName"),
            initiated: get_or_err!(m, "Initiated").parse()?,
            source_port: get_or_err!(m, "SourcePort").parse()?,
            destination_port: get_or_err!(m, "DestinationPort").parse()?,
        })
    }
}

fn from_intermediary_data<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<IntermediaryEventData>,
{
    let s: IntermediaryEventData = Deserialize::deserialize(deserializer)?;
    T::try_from(s).map_err(|_| SerdeError::custom("Failed to deserialize"))
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct Data {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Hash)]
pub struct IntermediaryEventData {
    #[serde(rename = "Data")]
    pub data: Vec<Data>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const NETWORK_EVENT: &str = r#"
    <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
        <System>
            <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}" />
            <EventID>3</EventID>
            <Version>5</Version>
            <Level>4</Level>
            <Task>3</Task>
            <Opcode>0</Opcode>
            <Keywords>0x8000000000000000</Keywords>
            <TimeCreated SystemTime="2017-04-28T22:12:23.657698300Z" />
            <EventRecordID>10953</EventRecordID>
            <Correlation />
            <Execution ProcessID="3216" ThreadID="3976" />
            <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
            <Computer>rfsH.lab.local</Computer>
            <Security UserID="S-1-5-18" />
        </System>
        <EventData>
            <Data Name="UtcTime">2017-04-28 22:12:22.557</Data>
            <Data Name="ProcessGuid">{A23EAE89-BD28-5903-0000-00102F345D00}</Data>
            <Data Name="ProcessId">13220</Data>
            <Data Name="Image">C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
            <Data Name="User">LAB\rsmith</Data>
            <Data Name="Protocol">tcp</Data>
            <Data Name="Initiated">true</Data>
            <Data Name="SourceIsIpv6">false</Data>
            <Data Name="SourceIp">192.168.1.250</Data>
            <Data Name="SourceHostname">rfsH.lab.local</Data>
            <Data Name="SourcePort">3328</Data>
            <Data Name="SourcePortName"></Data>
            <Data Name="DestinationIsIpv6">false</Data>
            <Data Name="DestinationIp">104.130.229.150</Data>
            <Data Name="DestinationHostname"></Data>
            <Data Name="DestinationPort">443</Data>
            <Data Name="DestinationPortName">https</Data>
        </EventData>
    </Event>
    "#;

    const FILE_CREATE: &str = r#"
        <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
        <System>
            <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}" />
            <EventID>11</EventID>
            <Version>2</Version>
            <Level>4</Level>
            <Task>11</Task>
            <Opcode>0</Opcode>
            <Keywords>0x8000000000000000</Keywords>
            <TimeCreated SystemTime="2017-05-13T19:44:55.314125100Z" />
            <EventRecordID>734181</EventRecordID>
            <Correlation />
            <Execution ProcessID="2848" ThreadID="3520" />
            <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
            <Computer>rfsH.lab.local</Computer>
            <Security UserID="S-1-5-18" />
        </System>
        <EventData>
            <Data Name="UtcTime">2017-05-13 19:44:55.313</Data>
            <Data Name="ProcessGuid">{A23EAE89-6237-5917-0000-0010300E6601}</Data>
            <Data Name="ProcessId">19200</Data>
            <Data Name="Image">C:\Windows\Microsoft.NET\Framework64\v4.0.30319\mscorsvw.exe</Data>
            <Data Name="TargetFilename">C:\Windows\assembly\NativeImages_v4.0.30319_64\Temp\4b00-0\AxImp.exe</Data>
            <Data Name="CreationUtcTime">2017-05-13 19:44:55.313</Data>
        </EventData>
        </Event>
    "#;

    const PROCESS_CREATE: &str = r#"
    <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
        <System>
            <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}" />
            <EventID>1</EventID>
            <Version>5</Version>
            <Level>4</Level>
            <Task>1</Task>
            <Opcode>0</Opcode>
            <Keywords>0x8000000000000000</Keywords>
            <TimeCreated SystemTime="2017-04-28T22:08:22.025812200Z" />
            <EventRecordID>9947</EventRecordID>
            <Correlation />
            <Execution ProcessID="3216" ThreadID="3964" />
            <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
            <Computer>rfsH.lab.local</Computer>
            <Security UserID="S-1-5-18" />
        </System>
        <EventData>
            <Data Name="UtcTime">2017-04-28 22:08:22.025</Data>
            <Data Name="ProcessGuid">{A23EAE89-BD56-5903-0000-0010E9D95E00}</Data>
            <Data Name="ProcessId">6228</Data>
            <Data Name="Image">C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
            <Data Name="CommandLine">"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" --type=utility --lang=en-US --no-sandbox --service-request-channel-token=F47498BBA884E523FA93E623C4569B94 --mojo-platform-channel-handle=3432 /prefetch:8</Data>
            <Data Name="CurrentDirectory">C:\Program Files (x86)\Google\Chrome\Application\58.0.3029.81\</Data>
            <Data Name="User">LAB\rsmith</Data>
            <Data Name="LogonGuid">{A23EAE89-B357-5903-0000-002005EB0700}</Data>
            <Data Name="LogonId">0x7eb05</Data>
            <Data Name="TerminalSessionId">1</Data>
            <Data Name="IntegrityLevel">Medium</Data>
            <Data Name="Hashes">SHA256=6055A20CF7EC81843310AD37700FF67B2CF8CDE3DCE68D54BA42934177C10B57</Data>
            <Data Name="ParentProcessGuid">{A23EAE89-BD28-5903-0000-00102F345D00}</Data>
            <Data Name="ParentProcessId">13220</Data>
            <Data Name="ParentImage">C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
            <Data Name="ParentCommandLine">"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" </Data>
        </EventData>
    </Event>
    "#;

    const HEADER: &'static str = r#"
        <System>
            <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}" />
            <EventID>1</EventID>
            <Version>5</Version>
            <Level>4</Level>
            <Task>1</Task>
            <Opcode>0</Opcode>
            <Keywords>0x8000000000000000</Keywords>
            <TimeCreated SystemTime="2017-04-28T22:08:22.025812200Z" />
            <EventRecordID>9947</EventRecordID>
            <Correlation />
            <Execution ProcessID="3216" ThreadID="3964" />
            <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
            <Computer>rfsH.lab.local</Computer>
            <Security UserID="S-1-5-18" />
        </System>
    "#;

    #[test]
    fn system() {
        serde_xml_rs::from_str::<System>(HEADER).unwrap();
    }

    #[test]
    fn process_create_event() {
        serde_xml_rs::from_str::<ProcessCreateEvent>(PROCESS_CREATE).unwrap();
    }

    #[test]
    fn file_create_event() {
        serde_xml_rs::from_str::<FileCreateEvent>(FILE_CREATE).unwrap();
    }

    #[test]
    fn network_event() {
        serde_xml_rs::from_str::<NetworkEvent>(NETWORK_EVENT).unwrap();
    }

    #[test]
    fn event_type() {
        assert!(Event::from_str(NETWORK_EVENT)
            .unwrap()
            .is_outbound_network());
        assert!(Event::from_str(FILE_CREATE).unwrap().is_file_create());
        assert!(Event::from_str(PROCESS_CREATE).unwrap().is_process_create());
    }
}
