use std::{
    collections::HashMap,
    io::Write,
    process::Command,
};

use nomad_client_gen::models;
use tempfile::NamedTempFile;

#[derive(Debug, thiserror::Error)]
pub enum ParseHclError {
    #[error("IOError {0:?}")]
    IOError(#[from] std::io::Error),
    #[error("DeserializeJsonError")]
    DeserializeJsonError(#[from] serde_json::Error),
}

pub type NomadVars = HashMap<String, String>;

/// Convert HCL2 + variables into JSON
fn parse_hcl2_to_json(hcl2: &str, vars: HashMap<String, String>) -> Result<String, std::io::Error> {
    // Write our static file to a temporary file
    let mut file = NamedTempFile::new()?;
    file.write_all(hcl2.as_bytes())?;
    let path = file.into_temp_path(); // Useful for when another process must read the file

    // Feed it to the Nomad binary.
    // Assume nomad is available on $PATH
    let mut command = Command::new("nomad");
    command.arg("job").arg("run").arg("-output");

    for (key, value) in &vars {
        command.arg("-var").arg(format!("{}={}", key, value));
    }

    // Specify filename
    command.arg(&path);

    let result = command.output()?;

    Ok(String::from_utf8(result.stdout).expect("Invalid UTF8"))
}

/// Convert HCL2 + variables into a Job
#[tracing::instrument(skip(hcl2, vars), err)]
pub fn parse_hcl2(hcl2: &str, vars: HashMap<String, String>) -> Result<models::Job, ParseHclError> {
    let job_json: String = parse_hcl2_to_json(hcl2, vars).map_err(ParseHclError::from)?;
    tracing::debug!(
        message = "The json is",
        json=?job_json,
    );
    // so we get back json that's like { "Job": {...} }
    // and ultimately, we want to deserialize that inner, {...} part
    // so I found an arbitrarily-chosen model that fits this structure
    // with a containing "Job"
    let job_parent: models::JobValidateRequest =
        serde_json::from_str(&job_json).map_err(ParseHclError::from)?;
    let job_box = job_parent.job.expect("Expected a Job here");
    Ok(*job_box)
}
