use std::borrow::Cow;

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use sysmon_parser::{
    event_data,
    system,
};
use uuid::Uuid;

#[test]

fn from_str() -> Result<(), Box<dyn std::error::Error>> {
    // TODO(inickles): use data from the file under 'tests/data' directory after we move this
    // package to its own repo
    // let xml = std::fs::read_to_string("tests/data/process_creation.xml")?;
    let xml = r#"<Event>
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
    </System>
    <EventData>
        <Data Name="RuleName">-</Data>
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
    </EventData>
</Event>
"#;

    let process_creation_event = sysmon_parser::SysmonEvent::from_str(xml)?;

    let system = process_creation_event.system;
    let process_create_event_data =
        event_data::ProcessCreateEventData::try_from(process_creation_event.event_data)?;

    assert_eq!(system.provider.name, Some(Cow::Borrowed("Linux-Sysmon")));
    assert_eq!(
        system.provider.guid,
        Some(Uuid::parse_str("ff032593-a8d3-4f13-b0d6-01fc615a0f97")?)
    );
    assert_eq!(system.provider.event_source_name, None);
    assert_eq!(system.event_id, system::EventId::ProcessCreation);
    assert_eq!(system.version, 5);
    assert_eq!(system.level, 4);
    assert_eq!(system.task, 1);
    assert_eq!(system.opcode, 0);
    assert_eq!(system.keywords, 0x8000000000000000);
    assert_eq!(
        system.time_created.system_time,
        "2022-01-04T19:54:15.665400000Z".parse::<DateTime<Utc>>()?
    );
    assert_eq!(system.event_record_id, 77);
    assert_eq!(system.correlation.activity_id, None);
    assert_eq!(system.correlation.related_activity_id, None);
    assert_eq!(system.execution.process_id, 49514);
    assert_eq!(system.execution.thread_id, 49514);
    assert_eq!(system.execution.processor_id, None);
    assert_eq!(system.execution.session_id, None);
    assert_eq!(system.execution.kernel_time, None);
    assert_eq!(system.execution.user_time, None);
    assert_eq!(system.execution.processor_time, None);
    assert_eq!(system.channel, Cow::Borrowed(r#"Linux-Sysmon/Operational"#));
    assert_eq!(system.computer, Cow::Borrowed("tux"));
    assert_eq!(system.security.user_id, Some(Cow::Borrowed("0")));

    assert_eq!(
        process_create_event_data.rule_name,
        Some(Cow::Borrowed("-"))
    );
    assert_eq!(
        process_create_event_data.utc_time,
        Utc.datetime_from_str("2022-01-04 19:54:15.661", event_data::UTC_TIME_FORMAT)?
    );
    assert_eq!(
        process_create_event_data.process_guid,
        Uuid::parse_str("49e2a5f6-a5e7-61d4-119e-dc77a5550000")?
    );
    assert_eq!(process_create_event_data.process_id, 49570);
    assert_eq!(
        process_create_event_data.image,
        Cow::Borrowed("/usr/bin/tr")
    );
    assert_eq!(
        process_create_event_data.file_version,
        Some(Cow::Borrowed("-"))
    );
    assert_eq!(
        process_create_event_data.description,
        Some(Cow::Borrowed("-"))
    );
    assert_eq!(process_create_event_data.product, Some(Cow::Borrowed("-")));
    assert_eq!(process_create_event_data.company, Some(Cow::Borrowed("-")));
    assert_eq!(
        process_create_event_data.original_file_name,
        Some(Cow::Borrowed("-"))
    );
    assert_eq!(
        process_create_event_data.command_line,
        Cow::Borrowed("tr [:upper:][:lower:]")
    );
    assert_eq!(
        process_create_event_data.current_directory,
        Cow::Borrowed("/root")
    );
    assert_eq!(process_create_event_data.user, Cow::Borrowed("root"));
    assert_eq!(
        process_create_event_data.logon_guid,
        Uuid::parse_str("49e2a5f6-0000-0000-0000-000000000000")?
    );
    assert_eq!(process_create_event_data.logon_id, 0);
    assert_eq!(process_create_event_data.terminal_session_id, 3);
    assert_eq!(
        process_create_event_data.integrity_level,
        Cow::Borrowed("no level")
    );
    assert_eq!(process_create_event_data.hashes, Cow::Borrowed("-"));
    assert_eq!(
        process_create_event_data.parent_process_guid,
        Uuid::parse_str("00000000-0000-0000-0000-000000000000")?
    );
    assert_eq!(process_create_event_data.parent_process_id, 49568);
    assert_eq!(process_create_event_data.parent_image, Cow::Borrowed("-"));
    assert_eq!(
        process_create_event_data.parent_command_line,
        Cow::Borrowed("-")
    );
    assert_eq!(
        process_create_event_data.parent_user,
        Some(Cow::Borrowed("-"))
    );
    assert_eq!(process_create_event_data.sequence_number, None);

    Ok(())
}
