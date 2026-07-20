use std::{
    any::Any,
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use gha_main::{GitHubActionResult, gha_main, gha_output};
use glob::glob;
use regex::{NoExpand, regex};
use serde::Deserialize;
use tracing::info;

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
    let update_provider: Box<dyn UpdateProvider>;
    let provider_data: Box<dyn Any>;

    match manifest_auto_updates {
        ManifestAutoUpdate::GithubReleases(ghreleases_data) => {
            update_provider = Box::new(Test {});
            provider_data = Box::new(ghreleases_data);
        }
        ManifestAutoUpdate::GithubTags(ghtags_data) => {
            update_provider = Box::new(Test2 {});
            provider_data = Box::new(ghtags_data);
        }
        ManifestAutoUpdate::Equinox(equinox_data) => {
            update_provider = Box::new(Test {});
            provider_data = Box::new(equinox_data);
        }
    }

    // Extract package version from PKGBUILD
    let pkgbuild_version = get_version_from_pkgbuild(&pkgbuild_path)?;

    // Get latest version using specified update provider
    let latest_version = update_provider.latest_version(&provider_data)?;
    info!(pkgbuild_version, latest_version);

    // Return early if current (PKGBUILD) version and latest version are equal
    if pkgbuild_version == latest_version {
        return Ok(());
    }

    // TODO: Check if pull request/branch already exists for latest (new) version

    // Update perms so that the builder user owns everything (prevent git complaining about unsafe directory)
    let cmd_status = Command::new("sudo")
        .args([
            "chown",
            "-R",
            "builder:builder",
            &current_dir()?.to_string_lossy(),
        ])
        .status()?;

    if !cmd_status.success() {
        return Err(anyhow!(
            "`sudo chown -R builder:builder $(pwd)` failed with exit status {cmd_status}"
        ));
    }

    // Retrieve PKGBUILD changes from update provider
    let update_data = update_provider.get_update_data(&provider_data)?;

    // Checkout Git to new branch (bot/{manifest.name}/{latest_version})
    info!("Checkout out new branch");
    let branch_name = format!("bot/{}/{}", manifest.name, latest_version);
    let cmd_status = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .status()?;

    if !cmd_status.success() {
        return Err(anyhow!(
            "`git checkout -b {branch_name}` failed with exit status {cmd_status}"
        ));
    }

    // Update PKGBUILD contents with new version (and reset pkgrel to 1)
    let pkgbuild_contents = fs::read_to_string(&pkgbuild_path)?;

    let pkgbuild_contents = regex!("(?m)^pkgver=.*$").replace(
        &pkgbuild_contents,
        NoExpand(&format!("pkgver={latest_version}")),
    );

    let pkgbuild_contents = regex!("(?m)^pkgrel=.*$")
        .replace(&pkgbuild_contents, NoExpand("pkgrel=1"))
        .to_string();

    // TODO: Update PKGBUILD source arrays with new sources (if provided by update provider)

    // Write updated PKGBUILD
    info!("Writing updated file");
    fs::write(&pkgbuild_path, pkgbuild_contents)?;

    // Use updpkgsums to update checksums in PKGBUILD
    if update_data.update_checksums {
        let cmd_status = Command::new("updpkgsums")
            .current_dir(manifest_path.parent().expect(
                "How can there be no parent? We've been operating with files inside of it.",
            ))
            .status()?;

        if !cmd_status.success() {
            return Err(anyhow!("`updpkgsums` failed with exit status {cmd_status}"));
        }
    }

    // Commit changes using Git (and custom Bot user)
    info!("Committing changes");
    let cmd_status = Command::new("git")
        .args(["add", &pkgbuild_path.to_string_lossy()])
        .status()?;

    if !cmd_status.success() {
        return Err(anyhow!(
            "`git add {}` failed with exit status {cmd_status}",
            pkgbuild_path.to_string_lossy()
        ));
    }

    let cmd_status = Command::new("git")
        .args([
            "commit",
            "-m",
            &format!("Update {} to {}", manifest.name, latest_version),
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

    if !cmd_status.success() {
        return Err(anyhow!(
            "`git commit -m \"Update {} to {latest_version}\"` failed with exit status {cmd_status}",
            manifest.name
        ));
    }

    info!("Pushing changes");
    let cmd_status = Command::new("git")
        .args(["push", "origin", &branch_name])
        .status()?;

    if !cmd_status.success() {
        return Err(anyhow!(
            "`git push origin {branch_name}` failed with exit status {cmd_status}"
        ));
    }

    // TODO: Create a new pull request
    info!("Opening PR");

    // Switch back to master branch
    let cmd_status = Command::new("git").args(["checkout", "master"]).status()?;

    if !cmd_status.success() {
        return Err(anyhow!(
            "`git checkout master` failed with exit status {cmd_status}"
        ));
    }

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

fn get_version_from_pkgbuild(pkgbuild_path: &Path) -> anyhow::Result<String> {
    let file_contents = fs::read_to_string(pkgbuild_path)?;

    let re = regex!("^pkgver=(.*)$");

    for line in file_contents.lines() {
        if let Some(version_captures) = re.captures(line) {
            return Ok(version_captures[1].to_string());
        }
    }

    Err(anyhow!("did not find version in PKGBUILD"))
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
