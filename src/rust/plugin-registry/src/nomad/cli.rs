use std::{
    collections::HashMap,
    io::Write,
    process::Command,
};

use nomad_client_gen::models;
use tempfile::NamedTempFile;

#[derive(Debug, thiserror::Error)]
pub enum NomadCliError {
    #[error("IOError {0:?}")]
    IOError(#[from] std::io::Error),
    #[error("DeserializeJsonError {0:?}")]
    DeserializeJsonError(#[from] serde_json::Error),
    #[error("NonzeroExitStatus stderr={0:?}")]
    NonzeroExitStatus(String),
}

pub type NomadVars = HashMap<&'static str, String>;

#[derive(Default)]
pub struct NomadCli {}

impl NomadCli {
    /// Convert HCL2 + variables into JSON
    #[tracing::instrument(skip(self, hcl2, vars), err)]
    fn parse_hcl2_to_json(&self, hcl2: &str, vars: NomadVars) -> Result<String, NomadCliError> {
        // Write our static file to a temporary file
        let mut file = NamedTempFile::new()?;
        file.write_all(hcl2.as_bytes())?;
        let path = file.into_temp_path(); // Useful for when another process must read the file

        // Feed it to the Nomad binary.
        // Assume nomad is available on $PATH
        let mut command = Command::new("nomad");
        command.args(["job", "run", "-output"]);

        for (key, value) in &vars {
            command.args(["-var", format!("{}={}", key, value).as_ref()]);
        }

        // Specify filename
        command.arg(&path);

        let result = command.output().map_err(|ioe| {
            if let std::io::ErrorKind::NotFound = ioe.kind() {
                panic!("No Nomad binary on path - make sure it's COPY'd in")
            }
            ioe
        })?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            return Err(NomadCliError::NonzeroExitStatus(stderr));
        };

        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        Ok(stdout)
    }

    /// Convert HCL2 + variables into a Job
    #[tracing::instrument(skip(self, hcl2, vars), err)]
    pub fn parse_hcl2(&self, hcl2: &str, vars: NomadVars) -> Result<models::Job, NomadCliError> {
        let job_json: String = self
            .parse_hcl2_to_json(hcl2, vars)
            .map_err(NomadCliError::from)?;

        // so we get back json that's like { "Job": {...} }
        // and ultimately, we want to deserialize that inner, {...} part
        // so I found an arbitrarily-chosen model that fits this structure
        // with a containing "Job"
        let job_parent: models::JobValidateRequest =
            serde_json::from_str(&job_json).map_err(NomadCliError::from)?;
        let job_box = job_parent.job.expect("Expected a Job here");
        let job = *job_box;
        // Quick sanity check that deserialization worked
        job.name
            .as_ref()
            .expect("Expected the job to have a Name field");
        Ok(job)
    }
}
