use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use itertools::intersperse_with;

pub fn clone_aur_repo(repo_name: &str, target_directory: impl AsRef<Path>) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clone", &format!("aur@aur.archlinux.org:{repo_name}.git")])
        .arg(target_directory.as_ref())
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while cloning repo: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

pub fn set_local_git_config(
    key: &str,
    value: &str,
    git_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["config", key, value])
        .current_dir(git_dir)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while setting git config key: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

pub fn generate_srcinfo(target_directory: impl AsRef<Path>) -> anyhow::Result<()> {
    let status = Command::new("bash")
        .args(["-c", "makepkg --printsrcinfo > .SRCINFO"])
        .current_dir(target_directory)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while running makepkg --printsrcinfo > .SRCINFO: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

pub fn git_add_files<I, S>(files: I, git_dir: impl AsRef<Path>) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new("git")
        .args(["add", "-f"])
        .args(files)
        .current_dir(git_dir)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while running makepkg --printsrcinfo > .SRCINFO: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

/// Run `git commit <message_args>`. Every entry in `message_args` will be preceded by `-m`.
pub fn git_commit<I, S>(message_args: I, git_dir: impl AsRef<Path>) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr> + From<&'static str>,
{
    let status = Command::new("git")
        .current_dir(git_dir)
        .args(["commit", "-m"])
        .args(intersperse_with(message_args, || S::from("-m")))
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while running git commit: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

pub fn git_push(git_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("push")
        .current_dir(git_dir)
        .status()?;

    if !status.success() {
        return Err(anyhow!(
            "error occurred while running git push: exit code {:?}",
            status.code()
        ));
    }

    Ok(())
}

pub fn git_modified_files(git_dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let cmd = Command::new("git")
        .args(["commit", "--short"])
        .current_dir(git_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "error occurred while running git commit --short: exit code {:?}",
            output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn pkgbuild_diff(git_dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let cmd = Command::new("git")
        .args(["diff", "HEAD~1", "PKGBUILD"])
        .current_dir(git_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "error occurred while running git diff HEAD~1 PKGBUILD: exit code {:?}",
            output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
