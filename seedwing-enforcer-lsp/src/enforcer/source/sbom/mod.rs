use crate::enforcer::{source::Source, Dependency};
use crate::highlight::Range;
use async_trait::async_trait;
use cyclonedx_bom::prelude::{Bom, Component};
use url::Url;

pub mod maven;

#[derive(Clone, Copy, Debug)]
pub enum CycloneDXFormat {
    Json,
    Xml,
}

#[derive(Clone, Copy, Debug)]
pub enum CycloneDXVersion {
    V1_3,
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
    CycloneDX {
        format: CycloneDXFormat,
        version: CycloneDXVersion,
    },
}

/// A generator which creates an SBOM for us
#[async_trait]
pub trait Generator {
    /// Generate an SBOM from the source
    async fn generate(&self) -> anyhow::Result<Output>;
    /// Find the range to highlight for the provided dependency
    fn highlight(&self, dependency: &Dependency) -> anyhow::Result<(Url, Range)>;
}

#[derive(Clone, Debug)]
pub struct Output {
    pub r#type: Type,
    pub content: Vec<u8>,
}

/// Generate a dependency list from an SBOM.
pub struct SBOM<G: Generator> {
    generator: G,
}

#[async_trait]
impl<G> Source for SBOM<G>
where
    G: Generator + Send + Sync,
{
    async fn scan(&self) -> anyhow::Result<Vec<Dependency>> {
        let Output { r#type, content } = self.generator.generate().await?;

        match r#type {
            Type::CycloneDX { format, version } => Self::from_cyclonedx(format, version, &content),
        }
    }

    fn highlight(&self, dependency: &Dependency) -> anyhow::Result<(Url, Range)> {
        self.generator.highlight(dependency)
    }
}

impl<G> SBOM<G>
where
    G: Generator,
{
    pub fn new(generator: G) -> Self {
        Self { generator }
    }

    fn from_cyclonedx(
        format: CycloneDXFormat,
        version: CycloneDXVersion,
        content: &[u8],
    ) -> anyhow::Result<Vec<Dependency>> {
        match (format, version) {
            (CycloneDXFormat::Json, CycloneDXVersion::V1_3) => {
                Self::from_bom(Bom::parse_from_json_v1_3(content)?)
            }
            (CycloneDXFormat::Xml, CycloneDXVersion::V1_3) => {
                Self::from_bom(Bom::parse_from_xml_v1_3(content)?)
            }
        }
    }

    /// Convert an SBOM into a vec of dependencies
    fn from_bom(bom: Bom) -> anyhow::Result<Vec<Dependency>> {
        bom.components
            .into_iter()
            .flat_map(|c| c.0.into_iter())
            .flat_map(Self::from_component)
            .collect()
    }

    /// Convert a component into a dependency
    fn from_component(component: Component) -> Option<anyhow::Result<Dependency>> {
        component.purl.map(|purl| {
            Ok::<_, anyhow::Error>(Dependency {
                purl: Url::parse(&purl.to_string())?,
            })
        })
    }
}
