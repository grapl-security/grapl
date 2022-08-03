use bytes::Bytes;

/// Send 1 line (well, event) at a time
pub fn get_36_eventlog_xml_separate_lines() -> Result<Vec<String>, std::io::Error> {
    let filename = "/test-fixtures/36_eventlog.xml"; // This path is created in rust/Dockerfile
    let content = std::fs::read_to_string(filename)?;
    Ok(content.lines().map(&str::to_owned).collect())
}

pub fn get_sysmon_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/sysmon-generator").map(Bytes::from)
}
