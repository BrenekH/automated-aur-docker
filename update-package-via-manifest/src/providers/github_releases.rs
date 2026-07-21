use std::{any::Any, env};

use anyhow::anyhow;
use serde::Deserialize;

use crate::providers::{UpdateData, UpdateProvider};

pub struct GitHubReleasesProvider {
    last_tag_html_link: Option<String>,
}

impl UpdateProvider for GitHubReleasesProvider {
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let data = manifest_data
            .downcast_ref::<GHReleasesData>()
            .ok_or(anyhow!(
                "Failed to downcast manifest_data to a GHReleasesData"
            ))?;

        let split: Vec<&str> = data.repo.split('/').collect();
        let owner = split
            .first()
            .ok_or(anyhow!("Couldn't extract owner from manifest"))?;
        let repo_name = split
            .get(1)
            .ok_or(anyhow!("Couldn't extract repo from manifest"))?;

        let response: serde_json::Value = ureq::get(format!(
            "https://api.github.com/repos/{owner}/{repo_name}/releases/latest"
        ))
        .header("Accept", "application/vnd.github+json")
        .header(
            "Authorization",
            format!("Bearer {}", env::var("INPUT_GITHUB-TOKEN")?),
        )
        .header("X-GitHub-Api-Version", "2026-03-10")
        .call()?
        .body_mut()
        .read_json()?;

        self.last_tag_html_link = response
            .get("html_url")
            .map(|v| v.as_str().map(|s| s.to_string()))
            .flatten();

        let mut tag_name = response
            .get("tag_name")
            .ok_or(anyhow!("`tag_name` not returned by GitHub API"))?
            .as_str()
            .ok_or(anyhow!("Couldn't read `tag_name` as a string"))?
            .to_string();

        // Remove a potential leading v (i.e. v1.0.0 -> 1.0.0)
        if tag_name.starts_with('v') {
            tag_name.remove(0);
        }

        Ok(tag_name)
    }

    fn get_update_data(&mut self, _: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        Ok(UpdateData {
            update_checksums: true,
            pr_content: self
                .last_tag_html_link
                .as_ref()
                .map(|link| format!("__GitHub Release Link:__ [{link}]({link})")),
            ..Default::default()
        })
    }
}

impl GitHubReleasesProvider {
    pub fn new() -> Self {
        Self {
            last_tag_html_link: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GHReleasesData {
    pub repo: String,
}
