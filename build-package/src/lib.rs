use std::{fs, path::Path};

use anyhow::anyhow;
use glob::glob;
use tracing::{debug, error, info};

use common::Manifest;

use crate::commands::{makepkg, namcap};

mod commands;

pub fn build_package(package_dir: impl AsRef<Path>) {
    build_pkg(package_dir).unwrap(); // TODO: Handle errors
}

fn build_pkg(package_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // Read and parse manifest
    let manifest_path = package_dir.as_ref().join(".aurmanifest.json");
    debug!(?manifest_path);

    let manifest_contents = fs::read_to_string(manifest_path)?;
    debug!(manifest_contents);
    let manifest: Manifest = serde_json::from_str(&manifest_contents)?;

    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Copy PKGBUILD + manifest.include to temp directory
    info!("Copying PKGBUILD and included files to temporary directory");
    for filename in ["PKGBUILD".to_string()]
        .iter()
        .chain(manifest.include.iter())
    {
        fs::copy(
            package_dir.as_ref().join(filename),
            temp_path.join(filename),
        )?;
    }

    // Install AUR deps with Paru
    if let Some(aur_deps) = manifest.aur_dependencies
        && !aur_deps.is_empty()
    {
        info!("Installing AUR dependencies: {}", aur_deps.join(", "));
        install_aur_deps(&aur_deps.iter().map(String::as_ref).collect::<Vec<&str>>())?;
    }

    // Build package with makepkg with no compression this time
    let _makepkg_output = makepkg(temp_path)?;

    // Run namcap against the PKGBUILD
    let _namcap_pkgbuild_output = namcap(temp_path.join("PKGBUILD"))?;

    // TODO: If makepkg returned an error, skip the remaining checks and return

    for entry in glob(
        temp_path
            .join("*.pkg.tar")
            .to_str()
            .ok_or(anyhow!("couldn't join glob pattern to temporary directory"))?,
    )
    .expect("Failed to read glob pattern")
    {
        match entry {
            Ok(_path) => {
                // TODO: Run namcap against all resulting packages

                // TODO: Copy all resulting packages to $GITHUB_WORKSPACE
            }
            Err(e) => error!(error=%e, "glob entry produced error"),
        }
    }

    // TODO: Create results text

    // TODO: Return results text and whether or not to fail the step

    Ok(())
}

fn install_aur_deps(deps: &[&str]) -> anyhow::Result<()> {
    if deps.is_empty() {
        return Ok(());
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let mut paru_args = vec![
                "--mflags",
                "--skippgpcheck",
                "--nocheck",
                "--noconfirm",
                "-Syu",
            ];
            paru_args.extend(deps);

            paru::run(&paru_args).await;
        });

    Ok(())
}
