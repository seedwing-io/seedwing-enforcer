use crate::config::{Config, ManifestType};
use crate::enforcer::source::cargo::CargoSource;
use crate::enforcer::source::sbom::maven::MavenGenerator;
use crate::enforcer::source::sbom::SBOM;
use crate::enforcer::source::Source;
use crate::utils::projects::{CARGO_FILE, MAVEN_FILE};
use anyhow::{bail, Result};
use std::io;
use std::path::PathBuf;

pub struct AutoSource {}

impl AutoSource {
    pub async fn find_source(
        path: impl Into<PathBuf>,
        config: Option<Config>,
    ) -> Result<Box<dyn Source>> {
        let root: PathBuf = path.into();

        if let Some(config) = config {
            if let Some(source_type) = config.enforcer.source {
                match source_type {
                    ManifestType::Cargo => Ok(cargo(root)),
                    ManifestType::Maven => Ok(maven(root)),
                }
            } else {
                autodetect(root)
            }
        } else {
            autodetect(root)
        }
    }
}

fn autodetect(path: PathBuf) -> Result<Box<dyn Source>> {
    if path.is_dir() {
        let cargo_path = path.join(CARGO_FILE);
        if cargo_path.exists() {
            return Ok(cargo(path));
        }

        let maven_path = path.join(MAVEN_FILE);
        if maven_path.exists() {
            return Ok(maven(path));
        }
    } else if path.ends_with(CARGO_FILE) {
        return Ok(cargo(path));
    } else if path.ends_with(MAVEN_FILE) {
        return Ok(maven(path));
    }
    bail!(io::ErrorKind::NotFound)
}

fn maven(root: impl Into<PathBuf>) -> Box<dyn Source> {
    Box::new(SBOM::new(MavenGenerator::new(root)))
}

fn cargo(root: impl Into<PathBuf>) -> Box<dyn Source> {
    Box::new(CargoSource::new(root))
}
