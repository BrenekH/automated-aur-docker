//! Collection of functions that call system commands (using [`Command`]).
//! Mainly consists of git operations, with some other useful items sprinkled in.

use std::{env::current_dir, fmt::Display, path::Path, process::Command};

use anyhow::anyhow;

macro_rules! validate_success {
    ($cmd_status:ident, $error:expr) => {
        if !$cmd_status.success() {
            return Err($error);
        } else {
            return Ok(());
        }
    };
}

pub fn ensure_folder_permissions() -> anyhow::Result<()> {
    let cmd_status = Command::new("sudo")
        .args([
            "chown",
            "-R",
            "builder:builder",
            &current_dir()?.to_string_lossy(),
        ])
        .status()?;

    validate_success!(
        cmd_status,
        anyhow!("`sudo chown -R builder:builder $(pwd)` failed with exit status {cmd_status}")
    )
}

pub fn checkout_new_branch(branch_name: impl AsRef<str> + Display) -> anyhow::Result<()> {
    let cmd_status = Command::new("git")
        .args(["checkout", "-b", branch_name.as_ref()])
        .status()?;

    validate_success!(
        cmd_status,
        anyhow!("`git checkout -b {branch_name}` failed with exit status {cmd_status}")
    )
}

pub fn update_package_checksums(manifest_path: &Path) -> anyhow::Result<()> {
    let cmd_status =
        Command::new("updpkgsums")
            .current_dir(manifest_path.parent().expect(
                "How can there be no parent? We've been operating with files inside of it.",
            ))
            .status()?;

    validate_success!(
        cmd_status,
        anyhow!("`updpkgsums` failed with exit status {cmd_status}")
    )
}

pub fn git_add_pkgbuild(pkgbuild_path: &Path) -> anyhow::Result<()> {
    let cmd_status = Command::new("git")
        .args(["add", &pkgbuild_path.to_string_lossy()])
        .status()?;

    validate_success!(
        cmd_status,
        anyhow!(
            "`git add {}` failed with exit status {cmd_status}",
            pkgbuild_path.to_string_lossy()
        )
    )
}

pub fn git_commit_new_version(name: &str, latest_version: &str) -> anyhow::Result<()> {
    let cmd_status = Command::new("git")
        .args([
            "commit",
            "-m",
            &format!("Update {} to {}", name, latest_version),
        ])
        .envs([
            ("GIT_AUTHOR_NAME", "github-actions[bot]"),
            (
                "GIT_AUTHOR_EMAIL",
                "41898282+github-actions[bot]@users.noreply.github.com",
            ),
            ("GIT_COMMITTER_NAME", "github-actions[bot]"),
            (
                "GIT_COMMITTER_EMAIL",
                "41898282+github-actions[bot]@users.noreply.github.com",
            ),
            (
                "EMAIL",
                "41898282+github-actions[bot]@users.noreply.github.com",
            ),
        ])
        .status()?;

    validate_success!(
        cmd_status,
        anyhow!(
            "`git commit -m \"Update {name} to {latest_version}\"` failed with exit status {cmd_status}",
        )
    )
}

pub fn git_push_branch(branch_name: &str) -> anyhow::Result<()> {
    let cmd_status = Command::new("git")
        .args(["push", "origin", branch_name])
        .status()?;

    validate_success!(
        cmd_status,
        anyhow!("`git push origin {branch_name}` failed with exit status {cmd_status}")
    )
}

pub fn git_checkout_master() -> anyhow::Result<()> {
    let cmd_status = Command::new("git").args(["checkout", "master"]).status()?;

    validate_success!(
        cmd_status,
        anyhow!("`git checkout master` failed with exit status {cmd_status}")
    )
}
