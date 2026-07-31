use std::{
    fs, io,
    path::Path,
    process::{Command, Stdio},
};

use common::Manifest;
use tracing::{debug, info};

#[derive(thiserror::Error, Debug)]
pub enum TestPkgError {
    /// An error occurred within the subprocess
    #[error("command error")]
    Cmd { output: String },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("couldn't parse manifest: {0}")]
    Parse(#[from] serde_json::Error),
}

pub fn test_package(package_dir: impl AsRef<Path>) -> Result<String, TestPkgError> {
    // Read manifest
    let manifest_path = package_dir.as_ref().join(".aurmanifest.json");
    debug!(?manifest_path);

    let manifest_contents = fs::read_to_string(manifest_path)?;
    debug!(manifest_contents);
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    // If test command is None, exit early
    let Some(test_cmd) = manifest.test_cmd else {
        return Ok("## Test Command:\n### Not Run (testCmd is null)".to_string());
    };

    // Run test command
    info!(command = test_cmd, "Running test command");
    let cmd = Command::new("bash")
        .arg("-c")
        .arg(&test_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let cmd_output = cmd.wait_with_output()?;

    // Format results
    let results_string = format!(
        "## Test Command:\n### Stdout:\n```\n{}\n```\n### Stderr:\n```\n{}\n```\n",
        String::from_utf8_lossy(&cmd_output.stdout),
        String::from_utf8_lossy(&cmd_output.stderr),
    );

    if cmd_output.status.success() {
        return Ok(results_string);
    }

    Err(TestPkgError::Cmd {
        output: results_string,
    })
}
