use std::{any::Any, collections::HashMap};

use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::providers::{UpdateData, UpdateProvider};

pub struct EquinoxProvider {
    latest_version: Option<String>,
}

impl UpdateProvider for EquinoxProvider {
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let data = manifest_data
            .downcast_ref::<EquinoxData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a EquinoxData"))?;

        let resp: CheckResp = ureq::post("https://update.equinox.io/check")
            .send_json(json!({
                "app_id": data.app_id,
                "channel": "stable",
                "current_sha256": "",
                "current_version": "0.0.0",
                "goarm": "",
                "os": "linux",
                "target_version": "",
                "arch": "amd64"
            }))?
            .body_mut()
            .read_json()?;

        Ok(resp.release.version)
    }

    fn get_update_data(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        let data = manifest_data
            .downcast_ref::<EquinoxData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a EquinoxData"))?;

        let latest_version = if let Some(v) = &self.latest_version {
            v
        } else {
            &self.latest_version(manifest_data)?
        };

        let mut update_data = UpdateData {
            update_checksums: true,
            ..Default::default()
        };
        let mut arch_map = HashMap::from([
            // The period at the end of each key is the beginning of the file extension in the URL.
            // It is used to ensure arm doesn't match the arm64 url.
            ("amd64.", &mut update_data.source_x86_64),
            ("386.", &mut update_data.source_i686),
            ("arm64.", &mut update_data.source_aarch64),
            ("arm.", &mut update_data.source_armv7h),
        ]);

        let urls = Self::get_linux_source_urls(&data.app_slug, latest_version)?;
        for url in urls {
            for (equinox_arch, data_pointer) in &mut arch_map {
                if url.contains(equinox_arch) {
                    **data_pointer = Some(vec![url.clone()]);
                }
            }
        }

        Ok(update_data)
    }
}

impl EquinoxProvider {
    pub fn new() -> Self {
        Self {
            latest_version: None,
        }
    }

    fn get_linux_source_urls(app_slug: &str, app_version: &str) -> anyhow::Result<Vec<String>> {
        let regx = Regex::new(&format!(
            r"https:\/\/bin\.equinox\.io\/.*\/.*-{}-linux-.*\.tar\.gz",
            app_version.replace('.', r"\.")
        ))?;

        let response = ureq::get(format!("https://dl.equinox.io/{app_slug}/stable/archive"))
            .call()?
            .body_mut()
            .read_to_string()?;

        Ok(regx
            .find_iter(&response)
            .map(|x| x.as_str().to_string())
            .collect())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquinoxData {
    #[serde(rename = "appID")]
    pub app_id: String,
    #[serde(rename = "appSlug")]
    pub app_slug: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CheckResp {
    release: CheckRelease,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CheckRelease {
    version: String,
}
