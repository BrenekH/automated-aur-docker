use std::any::Any;

use anyhow::anyhow;

use crate::{
    GHReleasesData,
    providers::{UpdateData, UpdateProvider},
};

pub struct GitHubReleasesProvider {}

impl UpdateProvider for GitHubReleasesProvider {
    fn latest_version(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<GHReleasesData>()
            .ok_or(anyhow!(
                "Failed to downcast manifest_data to a GHReleasesData"
            ))?;

        todo!()
    }

    fn get_update_data(&self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}

impl GitHubReleasesProvider {
    pub fn new() -> Self {
        Self {}
    }
}
