use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::anyhow;
use glob::glob;
use tracing::{debug, error, info};

use common::Manifest;

use crate::commands::{makepkg, namcap, sudo_copy_file};

mod commands;

pub enum BuildStatus {
    Failure,
    Success,
}

/// Returns the results text that should be presented and whether or not the operations
/// were a success.
pub fn build_package(package_dir: impl AsRef<Path>) -> (String, BuildStatus) {
    build_pkg(package_dir).unwrap() // TODO: Handle errors
}

/// Returns the results text that should be presented and whether or not the operations
/// were a success.
///
/// Unrecoverable/unskippable errors are surfaced using [`anyhow::Error`].
fn build_pkg(package_dir: impl AsRef<Path>) -> anyhow::Result<(String, BuildStatus)> {
    // String for results text and bool for failure status
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
    let makepkg_output = makepkg(temp_path)?;

    // Run namcap against the PKGBUILD
    let namcap_pkgbuild_output = namcap(temp_path.join("PKGBUILD"))?;

    let mut package_file_map: HashMap<PathBuf, Output> = HashMap::new();

    for entry in glob(
        temp_path
            .join("*.pkg.tar")
            .to_str()
            .ok_or(anyhow!("couldn't join glob pattern to temporary directory"))?,
    )
    .expect("Failed to read glob pattern")
    {
        match entry {
            Ok(built_pkg_path) => {
                // TODO: Instead of failing the whole job, perhaps an error message could be output to the Actions log and the comment text

                let package_file_name = built_pkg_path.file_name().ok_or(anyhow!(
                    "Couldn't get file name of built package path: {built_pkg_path:?}"
                ))?;

                info!("Running namcap against {}", built_pkg_path.display());
                let namcap_output = namcap(&built_pkg_path)?;

                info!(
                    "Copying built package ({}) to GITHUB_WORKSPACE",
                    package_file_name.display(),
                );
                sudo_copy_file(
                    &built_pkg_path,
                    PathBuf::from(env::var("GITHUB_WORKSPACE")?).join(package_file_name),
                )?;

                package_file_map.insert(built_pkg_path, namcap_output);
            }
            Err(e) => error!(error=%e, "glob entry produced error"),
        }
    }

    let mut result_text = String::with_capacity(
        (makepkg_output.stdout.len()
            + makepkg_output.stderr.len()
            + namcap_pkgbuild_output.stdout.len()
            + namcap_pkgbuild_output.stderr.len())
            * 2,
    );
    let mut build_status = BuildStatus::Success;

    // Only use makepkg output if it encountered an error
    if !makepkg_output.status.success() {
        build_status = BuildStatus::Failure;
        result_text.push_str(&gen_results_section("makepkg", &makepkg_output));
    }

    // PKGBUILD namcap
    result_text.push_str(&gen_results_section(
        "namcap PKGBUILD",
        &namcap_pkgbuild_output,
    ));

    // All .pkg.tar file namcaps
    for (path, output) in package_file_map {
        if !output.status.success() {
            build_status = BuildStatus::Failure;
        }

        result_text.push_str(&gen_results_section(
            &format!(
                "namcap {}",
                path.file_name()
                    .expect("How does this file not have a file name after we checked it")
                    .display()
            ),
            &output,
        ));
    }

    Ok((result_text, build_status))
}

fn gen_results_section(title: &str, output: &Output) -> String {
    let (stdout, stderr) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let mut pair_str = String::with_capacity(stdout.len() + stderr.len() + 1);

    for pair in [("Standard Output", stdout), ("Standard Error", stderr)] {
        pair_str.push_str(&format!("### {}:\n```\n{}\n```\n", pair.0, pair.1));
    }

    let exit_code_str = match output.status.code() {
        None | Some(0) => "",
        Some(code) => &format!(" (exit code: {code})"),
    };

    format!("## {title}{exit_code_str}:\n{pair_str}")
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
