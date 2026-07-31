use std::{fs, path::Path};

use common::Manifest;
use tracing::{debug, info};

use crate::commands::{
    clone_aur_repo, generate_srcinfo, git_add_files, git_commit, git_push, set_local_git_config
};

mod commands;

pub fn publish(package_dir: impl AsRef<Path>) {
    // Read manifest
    let manifest_path = package_dir.as_ref().join(".aurmanifest.json");
    debug!(?manifest_path);

    let manifest_contents = fs::read_to_string(manifest_path).unwrap();
    debug!(manifest_contents);
    let manifest: Manifest = serde_json::from_str(&manifest_contents).unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    info!("Cloning AUR repo");
    clone_aur_repo(&manifest.name, repo_path).unwrap();

    info!("Setting up git config");
    set_local_git_config("user.name", "BrenekH Automated AUR", repo_path).unwrap();
    set_local_git_config(
        "user.email",
        "brenekharrison+automatedaur@gmail.com",
        repo_path,
    )
    .unwrap();

    // TODO: Copy PKGBUILD and manifest.include files to cloned repo
    info!("Copying files to git repo");

    info!("Creating .SRCINFO");
    generate_srcinfo(repo_path).unwrap();

    // Write out proper .gitignore file (useful for new packages not yet uploaded).
    info!("Writing .gitignore");
    fs::write(
        repo_path.join(".gitignore"),
        "# Require every item to be force added\n*",
    )
    .unwrap();

    // Force-add all modified files to the repo (if .gitignore hasn't changed, force-adding it won't break anything, so it's hardcoded in)
    info!("Adding files");
    git_add_files(
        ["PKGBUILD", ".SRCINFO", ".gitignore"]
            .iter()
            .map(|f| f.to_string())
            .chain(manifest.include),
        repo_path,
    )
    .unwrap();

    // TODO: Commit files to cloned repo
    info!("Committing");
    git_commit(
        ["TODO", "REPLACE ME"].iter().map(|m| m.to_string()),
        repo_path,
    )
    .unwrap();

    info!("Pushing to AUR");
    git_push(repo_path).unwrap();
}
