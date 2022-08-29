#[test]
fn comments() -> eyre::Result<()> {
    // TODO(inickles): use data from the file under 'tests/data' directory after we move this
    // package to its own repo
    let xml = r#"
    <!-- example process creation event -->
    <!-- <Event><System></System><EventData></EventData></Event> -->
    <Event>
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
    <!-- example process creation event -->
    <!-- <Event><System></System><EventData></EventData></Event> -->
    <Event>
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
    <!-- fin -->"#;

    assert_eq!(sysmon_parser::parse_events(xml).count(), 2);

    assert!(sysmon_parser::parse_events(xml).all(|res| res.is_ok()));

    Ok(())
}
