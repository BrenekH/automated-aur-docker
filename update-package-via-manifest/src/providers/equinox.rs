use std::any::Any;

use anyhow::anyhow;

use crate::{
    EquinoxData,
    providers::{UpdateData, UpdateProvider},
};

pub struct EquinoxProvider {}

impl UpdateProvider for EquinoxProvider {
    fn latest_version(&self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String> {
        let _data = manifest_data
            .downcast_ref::<EquinoxData>()
            .ok_or(anyhow!("Failed to downcast manifest_data to a EquinoxData"))?;

        todo!()
    }

    fn get_update_data(&self, _manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData> {
        todo!()
    }
}

impl EquinoxProvider {
    pub fn new() -> Self {
        Self {}
    }
}
