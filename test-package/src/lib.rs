use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use common::Manifest;

#[expect(unused)]
pub fn test_package(package_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // Read manifest
    let manifest_path = package_dir.as_ref().join(".aurmanifest.json");

    let manifest_contents = fs::read_to_string(manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    // If test command is None, exit early
    let Some(test_cmd) = manifest.test_cmd else {
        return Ok(());
    };

    // Run test command
    let cmd = Command::new("bash")
        .arg("-c")
        .arg(&test_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let cmd_output = cmd.wait_with_output()?;

    // TODO: Format results

    // TODO: Return results

    todo!()
}
