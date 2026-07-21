use std::{any::Any, env};

use anyhow::anyhow;
use serde::Deserialize;

use crate::providers::{UpdateData, UpdateProvider};

pub struct GitHubTagsProvider {
    last_tag_html_link: Option<String>,
}

impl UpdateProvider for GitHubTagsProvider {
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let data = manifest_data
            .downcast_ref::<GHTagsData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a GHTagsData"))?;

        let split: Vec<&str> = data.repo.split('/').collect();
        let owner = split
            .first()
            .ok_or(anyhow!("Couldn't extract owner from manifest"))?;
        let repo_name = split
            .get(1)
            .ok_or(anyhow!("Couldn't extract repo from manifest"))?;

        let mut tags: Vec<TagResp> = ureq::get(format!(
            "https://api.github.com/repos/{owner}/{repo_name}/tags"
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

        tags.sort(); // There's no way sort actually works. This will need to be revisited.

        let mut latest_tag = tags
            .first()
            .ok_or(anyhow!("latest tag could not be found"))?
            .name
            .clone();

        self.last_tag_html_link = Some(format!(
            "https://github.com/{owner}/{repo_name}/releases/tag/{latest_tag}"
        ));

        // Remove a potential leading v (i.e. v1.0.0 -> 1.0.0)
        if latest_tag.starts_with('v') {
            latest_tag.remove(0);
        }

        Ok(latest_tag)
    }

    fn get_update_data(&mut self, _: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        Ok(UpdateData {
            update_checksums: true,
            pr_content: self
                .last_tag_html_link
                .as_ref()
                .map(|link| format!("__GitHub Tag Link:__ [{link}]({link})")),
            ..Default::default()
        })
    }
}

impl GitHubTagsProvider {
    pub fn new() -> Self {
        Self {
            last_tag_html_link: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GHTagsData {
    pub repo: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TagResp {
    name: String,
}
