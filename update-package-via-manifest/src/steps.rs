//! Collection of functions which perform discrete steps of the automatic updating process. 

use std::{any::Any, fs, path::Path};

use anyhow::anyhow;
use regex::{NoExpand, regex};
use tracing::info;

use crate::{
    ManifestAutoUpdate, Test, Test2, UpdateData, UpdateProvider,
    commands::{git_add_pkgbuild, git_commit_new_version, git_push_branch},
};

pub fn extract_provider_and_data(
    manifest_auto_updates: ManifestAutoUpdate,
) -> (Box<dyn UpdateProvider>, Box<dyn Any>) {
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

    (update_provider, provider_data)
}

pub fn get_version_from_pkgbuild(pkgbuild_path: &Path) -> anyhow::Result<String> {
    let file_contents = fs::read_to_string(pkgbuild_path)?;

    let re = regex!("^pkgver=(.*)$");

    for line in file_contents.lines() {
        if let Some(version_captures) = re.captures(line) {
            return Ok(version_captures[1].to_string());
        }
    }

    Err(anyhow!("did not find version in PKGBUILD"))
}

pub fn update_pkgbuild(
    pkgbuild_path: &Path,
    latest_version: &str,
    _update_data: &UpdateData,
) -> anyhow::Result<()> {
    // Update PKGBUILD contents with new version (and reset pkgrel to 1)
    let pkgbuild_contents = fs::read_to_string(pkgbuild_path)?;

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
    fs::write(pkgbuild_path, pkgbuild_contents)?;

    Ok(())
}

pub fn commit_and_push_changes(
    pkgbuild_path: &Path,
    manifest_name: &str,
    latest_version: &str,
    branch_name: &str,
) -> anyhow::Result<()> {
    info!("Committing changes");
    git_add_pkgbuild(pkgbuild_path)?;
    git_commit_new_version(manifest_name, latest_version)?;

    info!("Pushing changes");
    git_push_branch(branch_name)?;

    Ok(())
}

pub fn open_new_pull_request() -> anyhow::Result<()> {
    info!("Opening PR");
    todo!()
}
