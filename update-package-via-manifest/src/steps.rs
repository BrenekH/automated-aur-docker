//! Collection of functions which perform discrete steps of the automatic updating process.

use std::{any::Any, env, fs, path::Path};

use anyhow::anyhow;
use regex::{NoExpand, regex};
use serde_json::json;
use tracing::info;

use crate::{
    ManifestAutoUpdate, UpdateData, UpdateProvider,
    commands::{git_add_pkgbuild, git_commit_new_version, git_push_branch},
    providers::{EquinoxProvider, GitHubReleasesProvider, GitHubTagsProvider},
};

pub fn extract_provider_and_data(
    manifest_auto_updates: ManifestAutoUpdate,
) -> (Box<dyn UpdateProvider>, Box<dyn Any>) {
    let update_provider: Box<dyn UpdateProvider>;
    let provider_data: Box<dyn Any>;

    match manifest_auto_updates {
        ManifestAutoUpdate::GithubReleases(ghreleases_data) => {
            update_provider = Box::new(GitHubReleasesProvider::new());
            provider_data = Box::new(ghreleases_data);
        }
        ManifestAutoUpdate::GithubTags(ghtags_data) => {
            update_provider = Box::new(GitHubTagsProvider::new());
            provider_data = Box::new(ghtags_data);
        }
        ManifestAutoUpdate::Equinox(equinox_data) => {
            update_provider = Box::new(EquinoxProvider::new());
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

macro_rules! update_source_array {
    ($pkgbuild_contents:ident, $data:ident) => {
        if let Some(source_array) = &$data {
            $pkgbuild_contents = {
                static REGEX: regex::__private::Lazy<regex::Regex> =
                    regex::__private::Lazy::new(|| {
                        regex::Regex::new(concat!("(?m)^", stringify!($data), r"=\(.*\)$"))
                            .expect("invalid regex pattern")
                    });
                let re: &regex::Regex = &REGEX;
                re
            }
            .replace(
                &$pkgbuild_contents,
                NoExpand(&format!(
                    "{}={}",
                    stringify!($data),
                    slice_to_bash_array(source_array)
                )),
            )
            .to_string();
        }
    };
}

pub fn update_pkgbuild(
    pkgbuild_path: &Path,
    latest_version: &str,
    update_data: &UpdateData,
) -> anyhow::Result<()> {
    // Update PKGBUILD contents with new version (and reset pkgrel to 1)
    let pkgbuild_contents = fs::read_to_string(pkgbuild_path)?;

    let pkgbuild_contents = regex!("(?m)^pkgver=.*$").replace(
        &pkgbuild_contents,
        NoExpand(&format!("pkgver={latest_version}")),
    );

    let mut pkgbuild_contents = regex!("(?m)^pkgrel=.*$")
        .replace(&pkgbuild_contents, NoExpand("pkgrel=1"))
        .to_string();

    // Update PKGBUILD source arrays with new sources (if provided by update provider)
    let source = &update_data.source;
    let source_x86_64 = &update_data.source_x86_64;
    let source_i686 = &update_data.source_i686;
    let source_aarch64 = &update_data.source_aarch64;
    let source_armv7h = &update_data.source_armv7h;

    update_source_array!(pkgbuild_contents, source);
    update_source_array!(pkgbuild_contents, source_x86_64);
    update_source_array!(pkgbuild_contents, source_i686);
    update_source_array!(pkgbuild_contents, source_aarch64);
    update_source_array!(pkgbuild_contents, source_armv7h);

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

pub fn open_new_pull_request(
    branch_name: &str,
    package_name: &str,
    version: &str,
    pr_content: Option<&str>,
) -> anyhow::Result<()> {
    info!("Opening PR");

    let pr_content = if let Some(c) = pr_content {
        c.to_owned() + "\n\n"
    } else {
        String::new()
    };

    ureq::post("https://api.github.com/repos/BrenekH/automated-aur/pulls")
        .header("Accept", "application/vnd.github+json")
        .header(
            "Authorization",
            format!("Bearer {}", env::var("INPUT_GITHUB-TOKEN")?),
        )
        .header("X-GitHub-Api-Version", "2026-03-10")
        .send_json(json!({
            "head": branch_name,
            "base": "master",
            "maintainer_can_modify": true,
            "title": format!("Update {package_name} to {version}"),
            "body": pr_content + "_This PR was automatically opened by the [Automatic AUR system](https://github.com/BrenekH/automated-aur#README)._",
        }))?;

    Ok(())
}

/// Transforms a slice into a valid Bash array string.
///
/// ```rust
/// let items = vec!("a".to_string(), "2".to_string());
/// let bash_array = slice_to_bash_array(&items);
///
/// assert_eq!(bash_array, r#"("a" "2")"#);
/// ```
fn slice_to_bash_array(v: &[String]) -> String {
    let items: Vec<String> = v.iter().map(|s| format!("\"{s}\"")).collect();
    format!("({})", items.join(" "))
}

#[cfg(test)]
mod tests {
    use crate::steps::slice_to_bash_array;

    #[test]
    fn slice_to_bash_array_basic() {
        let items = vec!["a".to_string(), "2".to_string()];
        let bash_array = slice_to_bash_array(&items);

        assert_eq!(bash_array, r#"("a" "2")"#);

        let items = vec![];
        let bash_array = slice_to_bash_array(&items);

        assert_eq!(bash_array, "()");
    }
}
