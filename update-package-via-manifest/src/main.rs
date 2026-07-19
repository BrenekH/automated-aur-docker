use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use gha_main::{GitHubActionResult, gha_main, gha_output};
use glob::glob;
use regex::regex;
use serde::Deserialize;

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

    // Extract package version from PKGBUILD
    let pkgbuild_version = get_version_from_pkgbuild(&pkgbuild_path)?;

    // Get latest version using specified update provider

    // Return early if current (PKGBUILD) version and latest version are equal

    // Check if pull request/branch already exists for latest (new) version

    // Update perms so that the builder user owns everything (prevent git complaining about unsafe directory)

    // Retrieve PKGBUILD changes from update provider

    // Checkout Git to new branch (bot/{manifest.name}/{latest_version})

    // Update PKGBUILD contents with new version (and reset pkgrel to 1)

    // Update PKGBUILD source arrays with new sources (if provided by update provider)

    // Write updated PKGBUILD

    // Use updpkgsums to update checksums in PKGBUILD

    // Commit changes using Git (and custom Bot user)

    // Create a new pull request

    // Switch back to master branch

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
