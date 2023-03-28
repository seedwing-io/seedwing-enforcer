use crate::enforcer::{source::Source, Dependency};
use crate::highlight::Range;
use anyhow::anyhow;
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use url::Url;

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

#[deprecated(
    note = "This implementation ignores all kinds of cases, use the SBOM version in combination with the Maven source instead."
)]
pub struct MavenSource {
    root: PathBuf,
}

#[allow(deprecated)]
impl MavenSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[allow(deprecated)]
#[async_trait]
impl Source for MavenSource {
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>> {
        let content = fs::read_to_string(self.root.join("pom.xml"))?;

        let project: pom::Project = quick_xml::de::from_str(&content)?;

        Ok(project
            .dependencies
            .dependency
            .into_iter()
            .map(|d| d.try_into())
            .collect::<Result<Vec<Dependency>, _>>()?)
    }

    fn highlight(&self, _dependency: &Dependency) -> anyhow::Result<(Url, Range)> {
        Ok((
            Url::from_file_path(self.root.join("pom.xml"))
                .map_err(|()| anyhow!("Failed to build path URI"))?,
            Range::default(),
        ))
    }
}
