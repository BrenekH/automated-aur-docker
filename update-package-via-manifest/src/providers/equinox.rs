use std::any::Any;

use anyhow::anyhow;
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

        todo!()
    }
}

impl EquinoxProvider {
    pub fn new() -> Self {
        Self {
            latest_version: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquinoxData {
    #[serde(rename="appID")]
    pub app_id: String,
    #[serde(rename="appSlug")]
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
