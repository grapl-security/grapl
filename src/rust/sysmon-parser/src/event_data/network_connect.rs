use std::{
    borrow::Cow,
    net::IpAddr,
};

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

/// The network connection event logs TCP/UDP connections on the machine. It is disabled by
/// default. Each connection is linked to a process through the ProcessId and ProcessGUID fields.
/// The event also contains the source and destination host names IP addresses, port numbers and
/// IPv6 status.
///
/// <event name="SYSMONEVENT_NETWORK_CONNECT" value="3" level="Informational" template="Network connection detected" rulename="NetworkConnect" version="5" target="all">
///
/// <https://docs.microsoft.com/en-us/sysinternals/downloads/sysmon#event-id-3-network-connection>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, IntoOwned)]
pub struct NetworkConnectionEventData<'a> {
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

    /// <data name="Protocol" inType="win:UnicodeString" outType="xs:string" />
    pub protocol: Cow<'a, str>,

    /// <data name="Initiated" inType="win:Boolean" />
    pub initiated: bool,

    /// <data name="SourceIsIpv6" inType="win:Boolean" />
    pub source_is_ipv6: bool,

    /// <data name="SourceIp" inType="win:UnicodeString" outType="xs:string" />
    pub source_ip: IpAddr,

    /// <data name="SourceHostname" inType="win:UnicodeString" outType="xs:string" />
    pub source_hostname: Option<Cow<'a, str>>,

    /// <data name="SourcePort" inType="win:UInt16" />
    pub source_port: u16,

    /// <data name="SourcePortName" inType="win:UnicodeString" outType="xs:string" />
    pub source_port_name: Option<Cow<'a, str>>,

    /// <data name="DestinationIsIpv6" inType="win:Boolean" />
    pub destination_is_ipv6: bool,

    /// <data name="DestinationIp" inType="win:UnicodeString" outType="xs:string" />
    pub destination_ip: IpAddr,

    /// <data name="DestinationHostname" inType="win:UnicodeString" outType="xs:string" />
    pub destination_hostname: Option<Cow<'a, str>>,

    /// <data name="DestinationPort" inType="win:UInt16" />
    pub destination_port: u16,

    /// <data name="DestinationPortName" inType="win:UnicodeString" outType="xs:string" />
    pub destination_port_name: Option<Cow<'a, str>>,
}

impl<'a> NetworkConnectionEventData<'a> {
    pub(crate) fn try_from(tokenizer: &mut xmlparser::Tokenizer<'a>) -> Result<Self> {
        let mut rule_name = None;
        let mut sequence_number = None;
        let mut utc_time = None;
        let mut process_guid = None;
        let mut process_id = None;
        let mut image = None;
        let mut user = None;
        let mut protocol = None;
        let mut initiated = None;
        let mut source_is_ipv6 = None;
        let mut source_ip = None;
        let mut source_hostname = None;
        let mut source_port = None;
        let mut source_port_name = None;
        let mut destination_is_ipv6 = None;
        let mut destination_ip = None;
        let mut destination_hostname = None;
        let mut destination_port = None;
        let mut destination_port_name = None;

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
                            "User" => user = Some(util::unescape_xml(text)?),
                            "Protocol" => protocol = Some(util::unescape_xml(text)),
                            "Initiated" => initiated = Some(text.parse::<bool>()),
                            "SourceIsIpv6" => source_is_ipv6 = Some(text.parse::<bool>()),
                            "SourceIp" => source_ip = Some(text.parse::<IpAddr>()),
                            "SourceHostname" => source_hostname = Some(util::unescape_xml(text)?),
                            "SourcePort" => source_port = Some(text.parse::<u16>()),
                            "SourcePortName" => source_port_name = Some(util::unescape_xml(text)?),
                            "DestinationIsIpv6" => destination_is_ipv6 = Some(text.parse::<bool>()),
                            "DestinationIp" => destination_ip = Some(text.parse::<IpAddr>()),
                            "DestinationHostname" => {
                                destination_hostname = Some(util::unescape_xml(text)?)
                            }
                            "DestinationPort" => destination_port = Some(text.parse::<u16>()),
                            "DestinationPortName" => {
                                destination_port_name = Some(util::unescape_xml(text)?)
                            }
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
        let protocol = protocol.ok_or(Error::MissingField("Protocol"))??;
        let initiated = initiated.ok_or(Error::MissingField("Initiated"))??;
        let source_is_ipv6 = source_is_ipv6.ok_or(Error::MissingField("SourceIsIpv6"))??;
        let source_ip = source_ip.ok_or(Error::MissingField("SourceIp"))??;
        let source_port = source_port.ok_or(Error::MissingField("SourcePort"))??;
        let destination_is_ipv6 =
            destination_is_ipv6.ok_or(Error::MissingField("DestinationIsIpv6"))??;
        let destination_ip = destination_ip.ok_or(Error::MissingField("DestinationIp"))??;
        let destination_port = destination_port.ok_or(Error::MissingField("DestinationPort"))??;

        Ok(NetworkConnectionEventData {
            rule_name,
            sequence_number,
            utc_time,
            process_guid,
            process_id,
            image,
            user,
            protocol,
            initiated,
            source_is_ipv6,
            source_ip,
            source_hostname,
            source_port,
            source_port_name,
            destination_is_ipv6,
            destination_ip,
            destination_hostname,
            destination_port,
            destination_port_name,
        })
    }
}

impl<'a> TryFrom<EventData<'a>> for NetworkConnectionEventData<'a> {
    type Error = Error;

    fn try_from(event_data: EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::NetworkConnect(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("NetworkConnection")),
        }
    }
}

impl<'a, 'b: 'a> TryFrom<&'b EventData<'a>> for &NetworkConnectionEventData<'a> {
    type Error = Error;

    fn try_from(event_data: &'b EventData<'a>) -> Result<Self> {
        match event_data {
            EventData::NetworkConnect(event_data) => Ok(event_data),
            _ => Err(Error::ExpectEventType("NetworkConnection")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parse_network_connection_event() -> Result<()> {
        let xml = r#"
        <EventData>
            <Data Name='RuleName'></Data>
            <Data Name='UtcTime'>2018-12-08 20:39:24.541</Data>
            <Data Name='ProcessGuid'>{331D737B-28FF-5C0B-0000-001081250F00}</Data>
            <Data Name='ProcessId'>1772</Data>
            <Data Name='Image'>C:\Program Files (x86)\Google\Chrome\Application\chrome.exe</Data>
            <Data Name='User'>DESKTOP-34EOTDT\andy</Data>
            <Data Name='Protocol'>udp</Data>
            <Data Name='Initiated'>true</Data>
            <Data Name='SourceIsIpv6'>false</Data>
            <Data Name='SourceIp'>10.0.2.15</Data>
            <Data Name='SourceHostname'>DESKTOP-34EOTDT.attlocal.net</Data>
            <Data Name='SourcePort'>62977</Data>
            <Data Name='SourcePortName'></Data>
            <Data Name='DestinationIsIpv6'>false</Data>
            <Data Name='DestinationIp'>10.0.3.15</Data>
            <Data Name='DestinationHostname'></Data>
            <Data Name='DestinationPort'>1900</Data>
            <Data Name='DestinationPortName'>ssdp</Data>
        </EventData>"#;

        let mut tokenizer = xmlparser::Tokenizer::from(xml);
        let network_connection_event = NetworkConnectionEventData::try_from(&mut tokenizer)?;

        assert_eq!(
            network_connection_event,
            NetworkConnectionEventData {
                rule_name: None,
                sequence_number: None,
                utc_time: Utc.datetime_from_str("2018-12-08 20:39:24.541", UTC_TIME_FORMAT)?,
                process_guid: util::parse_win_guid_str("331D737B-28FF-5C0B-0000-001081250F00")?,
                process_id: 1772,
                image: Cow::Borrowed(
                    r#"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"#
                ),
                user: Some(Cow::Borrowed(r#"DESKTOP-34EOTDT\andy"#)),
                protocol: Cow::Borrowed("udp"),
                initiated: true,
                source_is_ipv6: false,
                source_ip: IpAddr::from_str("10.0.2.15")?,
                source_hostname: Some(Cow::Borrowed("DESKTOP-34EOTDT.attlocal.net")),
                source_port: 62977,
                source_port_name: None,
                destination_is_ipv6: false,
                destination_ip: IpAddr::from_str("10.0.3.15")?,
                destination_hostname: None,
                destination_port: 1900,
                destination_port_name: Some(Cow::Borrowed("ssdp")),
            }
        );

        Ok(())
    }
}
