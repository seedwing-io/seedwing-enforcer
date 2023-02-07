use crate::enforcer::source::sbom::{CycloneDXFormat, CycloneDXVersion, Generator, Output, Type};
use crate::enforcer::Dependency;
use crate::highlight::{Highlighter, Range};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use url::Url;

pub struct MavenGenerator {
    root: PathBuf,
}

#[async_trait]
impl Generator for MavenGenerator {
    async fn generate(&self) -> anyhow::Result<Output> {
        Ok(Output {
            r#type: Type::CycloneDX {
                format: CycloneDXFormat::Json,
                version: CycloneDXVersion::V1_3,
            },
            content: self.run()?,
        })
    }

    fn highlight(&self, dependency: &Dependency) -> anyhow::Result<(Url, Range)> {
        let content = fs::read_to_string(self.root.join("pom.xml"))?;
        let h = Highlighter::new(&content)?;
        let url = Url::from_file_path(&self.root.join("pom.xml"))
            .map_err(|()| anyhow!("Failed to create file URL"))?;

        // TODO: find actual dependency
        // TODO: find parent of transient dependency
        // then fall back to dependencies section
        // TODO: or full document

        let position = h.find(|p| p.tag_name().name() == "dependencies")?;

        Ok((url, position.unwrap_or_default()))
    }
}

impl MavenGenerator {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn run(&self) -> anyhow::Result<Vec<u8>> {
        let output = Command::new("mvn")
            .current_dir(&self.root)
            .args([
                "org.cyclonedx:cyclonedx-maven-plugin:2.7.1:makeAggregateBom",
                "-Dcyclonedx.skipAttach=true",
                "-DoutputFormat=json",
                "-DschemaVersion=1.3",
                "-Dcyclonedx.verbose=false",
            ])
            .output()?;

        log::info!("Status: {}", output.status);
        log::info!(
            "Output (stdout):\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        log::info!(
            "Output (stderr):\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        if !output.status.success() {
            bail!("Failed to run Maven SBOM generator");
        }

        Ok(fs::read(self.root.join("target").join("bom.json"))?)
    }
}
