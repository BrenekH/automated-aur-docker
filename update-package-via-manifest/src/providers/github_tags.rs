use std::any::Any;

use anyhow::anyhow;
use serde::Deserialize;

use crate::providers::{UpdateData, UpdateProvider};

pub struct GitHubTagsProvider {}

impl UpdateProvider for GitHubTagsProvider {
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<GHTagsData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a GHTagsData"))?;

        todo!()
    }

    fn get_update_data(&mut self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}

impl GitHubTagsProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GHTagsData {
    pub repo: String,
}
