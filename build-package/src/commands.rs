use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

use anyhow::anyhow;
use tracing::warn;

pub fn makepkg(dir_path: impl AsRef<Path>) -> anyhow::Result<Output> {
    let cmd = Command::new("makepkg")
        .args(["--syncdeps", "--nocolor", "--noconfirm", "--noprogressbar"])
        .env("PKGEXT", ".pkg.tar")
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output = cmd.wait_with_output()?;

    if output.status.success() {
        warn!(
            "error occurred while running makepkg --syncdeps --nocolor --noconfirm --noprogressbar: exit code {:?}",
            output.status.code()
        );
    }

    Ok(output)
}

pub fn namcap(file_path: impl AsRef<Path>) -> anyhow::Result<Output> {
    let cmd = Command::new("namcap")
        .arg("--info")
        .arg(file_path.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output = cmd.wait_with_output()?;

    if output.status.success() {
        warn!(
            "error occurred while running namcap --info \"{}\": exit code {:?}",
            file_path.as_ref().to_string_lossy(),
            output.status.code(),
        );
    }

    Ok(output)
}

pub fn sudo_copy_file(
    source_file: impl AsRef<Path>,
    target_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let command_string = format!(
        "sudo cp {} {}",
        source_file.as_ref().to_str().ok_or(anyhow!(
            "could not convert source file {} to str",
            source_file.as_ref().display(),
        ))?,
        target_file.as_ref().to_str().ok_or(anyhow!(
            "could not convert target file {} to str",
            target_file.as_ref().display(),
        ))?,
    );

    let status = Command::new("bash")
        .args(["-c", &command_string])
        .status()?;

    if status.success() {
        return Err(anyhow!(
            "error occurred while running {}: exit code {:?}",
            command_string,
            status.code()
        ));
    }

    Ok(())
}
