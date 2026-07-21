use std::{any::Any, collections::HashMap, fs, path::PathBuf};

use anyhow::anyhow;
use gha_main::{GitHubActionResult, gha_main, gha_output};
use glob::glob;
use serde::Deserialize;
use tracing::info;

use crate::commands::{
    checkout_new_branch, ensure_folder_permissions, get_remote_branches, git_checkout_master,
    update_package_checksums,
};
use crate::steps::{
    commit_and_push_changes, extract_provider_and_data, get_version_from_pkgbuild,
    open_new_pull_request, update_pkgbuild,
};

mod commands;
mod steps;

#[gha_main]
fn main() -> GitHubActionResult {
    // Find all packages in the pkgs directory (ie. pkgs/**/.aurmanifest.json)
    for entry in glob("./pkgs/**.aurmanifest.json").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                output_gha_command(
                    "debug",
                    HashMap::new(),
                    &format!("Evaluating {}", path.display()),
                );

                handle_manifest(path).expect("Failed to handle manifest");
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}

fn handle_manifest(manifest_path: PathBuf) -> anyhow::Result<()> {
    let pkgbuild_path = manifest_path
        .parent()
        .map(|p| p.join("PKGBUILD"))
        .ok_or(anyhow!("Couldn't create PKGBUILD path"))?;

    // Parse manifest into a Manifest struct
    let manifest_contents = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    let Some(manifest_auto_updates) = manifest.automatic_updates else {
        return Ok(());
    };

    // Create UpdateProvider
    let (update_provider, provider_data) = extract_provider_and_data(manifest_auto_updates);

    // Extract package version from PKGBUILD
    let pkgbuild_version = get_version_from_pkgbuild(&pkgbuild_path)?;

    // Get latest version using specified update provider
    let latest_version = update_provider.latest_version(&provider_data)?;
    info!(pkgbuild_version, latest_version);

    // Return early if current (PKGBUILD) version and latest version are equal
    if pkgbuild_version == latest_version {
        return Ok(());
    }

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

    // Update perms so that the builder user owns everything (prevent git complaining about unsafe directory)
    ensure_folder_permissions()?;

    // Retrieve PKGBUILD changes from update provider
    let update_data = update_provider.get_update_data(&provider_data)?;

    // Checkout Git to new branch (bot/{manifest.name}/{latest_version})
    info!("Checkout out new branch");
    let branch_name = format!("bot/{}/{}", manifest.name, latest_version);
    checkout_new_branch(&branch_name)?;

    update_pkgbuild(&pkgbuild_path, &latest_version, &update_data)?;

    // Use updpkgsums to update checksums in PKGBUILD
    if update_data.update_checksums {
        update_package_checksums(&manifest_path)?;
    }

    commit_and_push_changes(
        &pkgbuild_path,
        &manifest.name,
        &latest_version,
        &branch_name,
    )?;

    open_new_pull_request()?;

    // Switch back to master branch
    git_checkout_master()?;

    Ok(())
}

fn output_gha_command<S: std::fmt::Display>(command: S, parameters: HashMap<S, S>, value: S) {
    let formatted_params: Vec<String> = parameters
        .iter()
        .map(|(key, val)| format!("{key}={val}")) // TODO: Encode value (https://github.com/orgs/community/discussions/26736#discussioncomment-3253165)
        .collect();

    let param_str = if formatted_params.is_empty() {
        String::new()
    } else {
        formatted_params.join(",")
    };

    println!("::{command}{param_str}::{value}");
}

trait UpdateProvider {
    /// Returns the latest version available from the provider
    fn latest_version(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String>;

    /// Generates [`UpdateData`]
    fn get_update_data(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData>;
}

/// Collection of data to use when creating the final update Pull Request.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UpdateData {
    /// Whether or not to use `updpkgsums` to update the checksums in the `PKGBUILD`.
    update_checksums: bool,
    /// Optional additional context to include in the initial Pull Request message.
    pr_content: Option<String>,
    /// Optional new set of sources for the `source` array to replace in the `PKGBUILD`.
    source: Option<Vec<String>>,
    /// Optional new set of sources for the `source_x86_64` array to replace in the `PKGBUILD`.
    source_x86_64: Option<Vec<String>>,
    /// Optional new set of sources for the `source_i686` array to replace in the `PKGBUILD`.
    source_i686: Option<Vec<String>>,
    /// Optional new set of sources for the `source_aarch64` array to replace in the `PKGBUILD`.
    source_aarch64: Option<Vec<String>>,
    /// Optional new set of sources for the `source_armv7h` array to replace in the `PKGBUILD`.
    source_armv7h: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Manifest {
    name: String,
    test_cmd: Option<String>,
    include: Vec<String>,
    automatic_updates: Option<ManifestAutoUpdate>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ManifestAutoUpdate {
    GithubReleases(GHReleasesData),
    GithubTags(GHTagsData),
    Equinox(EquinoxData),
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GHReleasesData {
    repo: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GHTagsData {
    repo: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EquinoxData {
    app_id: String,
    app_slug: String,
}

struct Test {}

impl UpdateProvider for Test {
    fn latest_version(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<GHReleasesData>()
            .ok_or(anyhow!(
                "Failed to downcast manifest_data to a GHReleasesData"
            ))?;

        todo!()
    }

    fn get_update_data(&self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}

struct Test2 {}

impl UpdateProvider for Test2 {
    fn latest_version(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<GHTagsData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a GHTagsData"))?;

        todo!()
    }

    fn get_update_data(&self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}
