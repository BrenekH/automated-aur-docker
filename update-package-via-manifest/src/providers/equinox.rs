use std::any::Any;

use anyhow::anyhow;
use serde::Deserialize;

use crate::providers::{UpdateData, UpdateProvider};

pub struct EquinoxProvider {}

impl UpdateProvider for EquinoxProvider {
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<EquinoxData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a EquinoxData"))?;

        todo!()
    }

    fn get_update_data(&mut self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}

impl EquinoxProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquinoxData {
    pub app_id: String,
    pub app_slug: String,
}
