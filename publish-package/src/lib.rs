use std::{env, fs, path::Path, process::exit};

use anyhow::anyhow;
use regex::{RegexSet, regex};
use serde::Deserialize;
use tracing::{debug, error, info};

use common::Manifest;

use crate::commands::{
    clone_aur_repo, generate_srcinfo, git_add_files, git_commit, git_modified_files, git_push,
    pkgbuild_diff, set_local_git_config,
};

mod commands;

pub fn publish(package_dir: impl AsRef<Path>) {
    match publish_result(package_dir) {
        Ok(_) => {}
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    }
}

fn publish_result(package_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // Read manifest
    let manifest_path = package_dir.as_ref().join(".aurmanifest.json");
    debug!(?manifest_path);

    let manifest_contents = fs::read_to_string(manifest_path)?;
    debug!(manifest_contents);
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path();

    info!("Cloning AUR repo");
    clone_aur_repo(&manifest.name, repo_path)?;

    info!("Setting up git config");
    set_local_git_config("user.name", "BrenekH Automated AUR", repo_path)?;
    set_local_git_config(
        "user.email",
        "brenekharrison+automatedaur@gmail.com",
        repo_path,
    )?;

    info!("Copying PKGBUILD and included files to git repo");
    for filename in ["PKGBUILD".to_string()]
        .iter()
        .chain(manifest.include.iter())
    {
        fs::copy(
            package_dir.as_ref().join(filename),
            repo_path.join(filename),
        )?;
    }

    info!("Creating .SRCINFO");
    generate_srcinfo(repo_path)?;

    // Write out proper .gitignore file (useful for new packages not yet uploaded).
    info!("Writing .gitignore");
    fs::write(
        repo_path.join(".gitignore"),
        "# Require every item to be force added\n*",
    )?;

    // Force-add all modified files to the repo (if .gitignore hasn't changed, force-adding it won't break anything, so it's hardcoded in)
    info!("Adding files");
    git_add_files(
        ["PKGBUILD", ".SRCINFO", ".gitignore"]
            .iter()
            .map(|f| f.to_string())
            .chain(manifest.include),
        repo_path,
    )?;

    info!("Committing");
    git_commit(
        generate_commit_message(repo_path)?
            .iter()
            .map(|m| m.to_string()),
        repo_path,
    )?;

    info!("Pushing to AUR");
    git_push(repo_path)?;

    Ok(())
}

/// Create a commit message based on the PR information and changed files.
fn generate_commit_message(git_repo_dir: impl AsRef<Path>) -> anyhow::Result<Vec<String>> {
    let event_filepath = env::var("GITHUB_EVENT_PATH")?;
    let file_data = fs::read_to_string(event_filepath)?;
    let github_event: GithubEvent = serde_json::from_str(&file_data)?;
    let pr_title = github_event.pull_request.title;
    let pr_num = github_event.pull_request.number;

    let bot_commit_msg = format!(
        "Automatically committed from https://github.com/BrenekH/automated-aur/pull/{pr_num}."
    );

    let changed_files_str = git_modified_files(&git_repo_dir)?;

    if pr_title.starts_with("Update")
        && changed_files_str.contains("PKGBUILD")
        && let Ok(message) = upstream_update_commit_msg(&git_repo_dir, &bot_commit_msg)
    {
        return Ok(message);
    }

    Ok(vec![pr_title, bot_commit_msg])
}

/// Creates a commit message of the format `Update to pkgver-1` when the changes
/// are a version bump because upstream updated.
fn upstream_update_commit_msg(
    git_repo_dir: impl AsRef<Path>,
    bot_commit_msg: &str,
) -> anyhow::Result<Vec<String>> {
    let pkgbuild_diff = pkgbuild_diff(&git_repo_dir)?;

    let set = RegexSet::new([r"-pkgver=.*\n+pkgver=.*", r"-pkgrel=.*\n+pkgrel=.*"])?;

    if !set.is_match(&pkgbuild_diff) {
        return Err(anyhow!("pkgver and pkgrel don't have pending changes"));
    }

    let pkgbuild_contents = fs::read_to_string(git_repo_dir.as_ref().join("PKGBUILD"))?;

    let Some(captures) = regex!(r"pkgver=(.*)").captures(&pkgbuild_contents) else {
        return Err(anyhow!("couldn't find pkgver"));
    };
    let pkgver = &captures[1];

    let Some(captures) = regex!(r"pkgrel=(.*)").captures(&pkgbuild_contents) else {
        return Err(anyhow!("couldn't find pkgrel"));
    };
    let pkgrel = &captures[1];

    // If the pkgrel was bumped higher than one, then we'd rather the PR title be
    // the commit message, because it was a packaging change, not a simple update bump.
    if pkgrel == "1" {
        Ok(vec![
            format!("Update to {pkgver}-{pkgrel}"),
            bot_commit_msg.to_string(),
        ])
    } else {
        Err(anyhow!("pkgrel updated to value >= 2"))
    }
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GithubEvent {
    pull_request: GHEventPR,
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GHEventPR {
    title: String,
    number: usize,
}
