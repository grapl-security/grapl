#[test]
#[ignore]
// This is test is ignored for now as it's currently known to fail due to relying on an XML 1.0
// parser and Sysmon for Linux, which uses libxml2, is including control characters, which are not
// valid for 1.0.
fn control_char() -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string("tests/data/event_with_control_char.xml")?;

    assert!(sysmon_parser::parse_events(&xml).all(|res| res.is_ok()));

    Ok(())
}
