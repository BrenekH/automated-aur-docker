use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Manifest {
    pub name: String,
    #[serde(rename = "testCmd")]
    pub test_cmd: Option<String>,
    pub include: Vec<String>,
    #[serde(rename = "automaticUpdates")]
    pub automatic_updates: Option<ManifestAutoUpdate>,
    #[serde(rename = "aurDeps")]
    pub aur_dependencies: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "type")]
pub enum ManifestAutoUpdate {
    #[serde(rename = "github-releases")]
    GithubReleases(GHReleasesData),
    #[serde(rename = "github-tags")]
    GithubTags(GHTagsData),
    #[serde(rename = "equinox")]
    Equinox(EquinoxData),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GHReleasesData {
    pub repo: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GHTagsData {
    pub repo: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquinoxData {
    #[serde(rename = "appID")]
    pub app_id: String,
    #[serde(rename = "appSlug")]
    pub app_slug: String,
}

#[cfg(test)]
mod tests {
    use super::{EquinoxData, GHReleasesData, GHTagsData, Manifest, ManifestAutoUpdate};

    #[test]
    fn deserialize_manifest_basic() {
        let text = r#"{
	"name": "auto-editor",
	"testCmd": "auto-editor --version",
	"include": [],
	"aurDeps": []
}"#;
        let manifest: Manifest = serde_json::from_str(text).expect("failed to deserialize");
        assert_eq!(
            manifest,
            Manifest {
                name: "auto-editor".into(),
                test_cmd: Some("auto-editor --version".into()),
                include: vec![],
                automatic_updates: None,
                aur_dependencies: Some(vec![]),
            }
        );
    }

    #[test]
    fn deserialize_manifest_gh_releases() {
        let text = r#"{
	"name": "auto-editor",
	"testCmd": "auto-editor --version",
	"include": ["extra-file"],
    "automaticUpdates": {
        "type": "github-releases",
        "repo": "WyattBlue/auto-editor"
    }
}"#;
        let manifest: Manifest = serde_json::from_str(text).expect("failed to deserialize");
        assert_eq!(
            manifest,
            Manifest {
                name: "auto-editor".into(),
                test_cmd: Some("auto-editor --version".into()),
                include: vec!["extra-file".into()],
                automatic_updates: Some(ManifestAutoUpdate::GithubReleases(GHReleasesData {
                    repo: "WyattBlue/auto-editor".into()
                })),
                aur_dependencies: None,
            }
        );
    }

    #[test]
    fn deserialize_manifest_gh_tags() {
        let text = r#"{
	"name": "auto-editor",
	"include": [],
    "automaticUpdates": {
        "type": "github-tags",
        "repo": "WyattBlue/auto-editor"
    }
}"#;
        let manifest: Manifest = serde_json::from_str(text).expect("failed to deserialize");
        assert_eq!(
            manifest,
            Manifest {
                name: "auto-editor".into(),
                test_cmd: None,
                include: vec![],
                automatic_updates: Some(ManifestAutoUpdate::GithubTags(GHTagsData {
                    repo: "WyattBlue/auto-editor".into()
                })),
                aur_dependencies: None,
            }
        );
    }

    #[test]
    fn deserialize_manifest_equinox() {
        let text = r#"{
	"name": "ngrok",
	"include": [],
    "automaticUpdates": {
        "type": "equinox",
		"appID": "app_c3U4eZcDbjV",
		"appSlug": "ngrok/ngrok-v3"
    }
}"#;
        let manifest: Manifest = serde_json::from_str(text).expect("failed to deserialize");
        assert_eq!(
            manifest,
            Manifest {
                name: "ngrok".into(),
                test_cmd: None,
                include: vec![],
                automatic_updates: Some(ManifestAutoUpdate::Equinox(EquinoxData {
                    app_id: "app_c3U4eZcDbjV".into(),
                    app_slug: "ngrok/ngrok-v3".into()
                })),
                aur_dependencies: None,
            }
        );
    }
}
