use crate::enforcer::{source::Source, Dependency};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

mod pom;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MavenDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,

    pub r#type: Option<String>,

    pub classifier: Option<String>,
}

impl From<pom::Dependency> for MavenDependency {
    fn from(value: pom::Dependency) -> Self {
        Self {
            group_id: value.group_id,
            artifact_id: value.artifact_id,
            version: value.version,
            r#type: value.r#type,
            classifier: None,
        }
    }
}

pub struct MavenSource {
    root: PathBuf,
}

impl MavenSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[async_trait]
impl Source for MavenSource {
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>> {
        let content = fs::read_to_string(self.root.join("pom.xml")).await?;

        let project: pom::Project = quick_xml::de::from_str(&content)?;

        Ok(project
            .dependencies
            .dependency
            .into_iter()
            .map(|d| d.try_into())
            .collect::<Result<Vec<Dependency>, _>>()?)
    }
}
