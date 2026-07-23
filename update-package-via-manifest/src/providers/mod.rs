use std::any::Any;

mod equinox;
mod github_releases;
mod github_tags;

pub use equinox::{EquinoxData, EquinoxProvider};
pub use github_releases::{GHReleasesData, GitHubReleasesProvider};
pub use github_tags::{GHTagsData, GitHubTagsProvider};

pub trait UpdateProvider {
    /// Returns the latest version available from the provider
    fn latest_version(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<String>;

    /// Generates [`UpdateData`]
    fn get_update_data(&mut self, manifest_data: &Box<dyn Any>) -> anyhow::Result<UpdateData>;
}

/// Collection of data to use when creating the final update Pull Request.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UpdateData {
    /// Whether or not to use `updpkgsums` to update the checksums in the `PKGBUILD`.
    pub update_checksums: bool,
    /// Optional additional context to include in the initial Pull Request message.
    pub pr_content: Option<String>,
    /// Optional new set of sources for the `source` array to replace in the `PKGBUILD`.
    pub source: Option<Vec<String>>,
    /// Optional new set of sources for the `source_x86_64` array to replace in the `PKGBUILD`.
    pub source_x86_64: Option<Vec<String>>,
    /// Optional new set of sources for the `source_i686` array to replace in the `PKGBUILD`.
    pub source_i686: Option<Vec<String>>,
    /// Optional new set of sources for the `source_aarch64` array to replace in the `PKGBUILD`.
    pub source_aarch64: Option<Vec<String>>,
    /// Optional new set of sources for the `source_armv7h` array to replace in the `PKGBUILD`.
    pub source_armv7h: Option<Vec<String>>,
}
