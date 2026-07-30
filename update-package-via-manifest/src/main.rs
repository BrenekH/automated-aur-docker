#![warn(clippy::pedantic)]

use std::{fs, path::PathBuf};

use anyhow::anyhow;
use common::Manifest;
use glob::glob;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use common::gha_subscriber::GHALayer;

use crate::commands::{
    checkout_new_branch, ensure_folder_permissions, get_remote_branches, git_checkout_master,
    update_package_checksums,
};
use crate::providers::{UpdateData, UpdateProvider};
use crate::steps::{
    commit_and_push_changes, extract_provider_and_data, get_version_from_pkgbuild,
    open_new_pull_request, update_pkgbuild,
};

mod commands;
mod providers;
mod steps;

fn main() {
    tracing_subscriber::registry()
        .with(GHALayer {})
        .with(LevelFilter::DEBUG)
        .init();

    // Find all packages in the pkgs directory (ie. pkgs/**/.aurmanifest.json)
    for entry in glob("./pkgs/**/.aurmanifest.json").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                debug!("Evaluating {}", path.display());

                if let Err(e) = handle_manifest(&path) {
                    error!(path=%path.display(), error=%e, title="Failed to handle manifest", "error while handling manifest");
                }
            }
            Err(e) => error!(error=%e, "glob entry produced error"),
        }
    }
}

fn handle_manifest(manifest_path: &PathBuf) -> anyhow::Result<()> {
    let pkgbuild_path = manifest_path
        .parent()
        .map(|p| p.join("PKGBUILD"))
        .ok_or(anyhow!("Couldn't create PKGBUILD path"))?;

    // Parse manifest into a Manifest struct
    let manifest_contents = fs::read_to_string(manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    let Some(manifest_auto_updates) = manifest.automatic_updates else {
        return Ok(());
    };

    info!("Checking for updates for {}", manifest.name);

    // Create UpdateProvider
    let (mut update_provider, provider_data) = extract_provider_and_data(manifest_auto_updates);

    // Extract package version from PKGBUILD
    let pkgbuild_version = get_version_from_pkgbuild(&pkgbuild_path)?;

    // Get latest version using specified update provider
    let latest_version = update_provider.latest_version(&provider_data)?;
    info!(
        "PKGBUILD Version" = pkgbuild_version,
        "Latest Version" = latest_version
    );

    // Return early if current (PKGBUILD) version and latest version are equal
    if pkgbuild_version == latest_version {
        return Ok(());
    }

    // Update perms so that the builder user owns everything (prevent git complaining about unsafe directory)
    ensure_folder_permissions()?;

    // Check if pull request/branch already exists for latest (new) version
    let remote_branches = get_remote_branches()?;
    for line in remote_branches.lines() {
        if line.contains(&format!("{}/{}", manifest.name, latest_version)) {
            info!(
                "Update for {}@{} has already been pushed",
                manifest.name, latest_version
            );
            return Ok(());
        }
    }

    // Retrieve PKGBUILD changes from update provider
    let update_data = update_provider.get_update_data(&provider_data)?;

    // Checkout Git to new branch (bot/{manifest.name}/{latest_version})
    let branch_name = format!("bot/{}/{}", manifest.name, latest_version);
    info!(branch_name, "Checking out new branch");
    checkout_new_branch(&branch_name)?;

    update_pkgbuild(&pkgbuild_path, &latest_version, &update_data)?;

    // Use updpkgsums to update checksums in PKGBUILD
    if update_data.update_checksums {
        update_package_checksums(manifest_path)?;
    }

    commit_and_push_changes(
        &pkgbuild_path,
        &manifest.name,
        &latest_version,
        &branch_name,
    )?;

    open_new_pull_request(
        &branch_name,
        &manifest.name,
        &latest_version,
        update_data.pr_content.as_deref(),
    )?;

    // Switch back to master branch
    git_checkout_master()?;

    Ok(())
}
