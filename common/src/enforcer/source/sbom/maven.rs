use crate::utils::projects::MAVEN_FILE;
use crate::{
    enforcer::{
        source::sbom::{CycloneDXFormat, CycloneDXVersion, Generator, Output, Type},
        Dependency,
    },
    highlight::{Highlighter, Range},
};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use std::{fs, path::PathBuf, process::Command};
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

    fn highlight(&self, _: &Dependency) -> anyhow::Result<(Url, Range)> {
        let content = fs::read_to_string(self.root.join(MAVEN_FILE))?;
        let h = Highlighter::new(&content)?;
        let url = Url::from_file_path(self.root.join(MAVEN_FILE))
            .map_err(|()| anyhow!("Failed to create file URL"))?;

        // TODO: find actual dependency
        // TODO: find parent of transient dependency

        let position = h
            // find the main dependencies section
            .find_with(|doc| {
                doc.root_element()
                    .children()
                    .find(|p| p.tag_name().name() == "dependencies")
            })
            // then fall back to dependencies section
            .or_else(|_| {
                h.find_with(|doc| {
                    doc.root_element()
                        .children()
                        .find(|p| p.tag_name().name() == "dependencyManagement")
                        .and_then(|d| d.children().find(|p| p.tag_name().name() == "dependencies"))
                })
            })?
            // or the full document
            .unwrap_or_else(|| h.full_range());

        Ok((url, position))
    }
}

#[cfg(not(target_os = "windows"))]
const MVN_WRAPPER: &str = "mvnw";
#[cfg(target_os = "windows")]
const MVN_WRAPPER: &str = "mvnw.cmd";

impl MavenGenerator {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn find_mvn(&self) -> anyhow::Result<PathBuf> {
        if let Ok(mvn) = which::which("mvn") {
            return Ok(mvn);
        }

        let mvnw = self.root.join(MVN_WRAPPER);
        log::debug!("Checking existence: {}", mvnw.display());
        if mvnw.exists() {
            return Ok(mvnw);
        }

        Err(anyhow!("could not find 'mvn' command"))
    }

    fn run(&self) -> anyhow::Result<Vec<u8>> {
        let mvn = self.find_mvn()?;

        let output = Command::new(mvn)
            .current_dir(&self.root)
            .args([
                "org.cyclonedx:cyclonedx-maven-plugin:2.7.5:makeAggregateBom",
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
