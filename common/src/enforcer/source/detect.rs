use crate::config::{try_load, ManifestType};
use crate::enforcer::source::cargo::CargoSource;
use crate::enforcer::source::maven::MavenSource;
use crate::enforcer::source::Source;
use anyhow::{bail, Result};
use std::io;
use std::path::PathBuf;

pub struct AutoSource {}

impl AutoSource {
    pub async fn find_source(path: impl Into<PathBuf>) -> Result<Box<dyn Source>> {
        let root: PathBuf = path.into();
        let config = try_load(root.clone().as_path())
            .await
            .map(|r| r.expect("Error loading config !"));

        if let Some(config) = config {
            if let Some(source_type) = config.enforcer.source {
                match source_type {
                    ManifestType::Cargo => Ok(Box::new(CargoSource::new(root))),
                    ManifestType::Maven => Ok(Box::new(MavenSource::new(root))),
                }
            } else {
                autodetect(root)
            }
        } else {
            autodetect(root)
        }
    }
}

const CARGO_FILE: &str = "Cargo.taml";
const MAVEN_FILE: &str = "pom.xml";

fn autodetect(path: PathBuf) -> Result<Box<dyn Source>> {
    if path.is_dir() {
        let cargo_path = path.join(CARGO_FILE);
        if cargo_path.exists() {
            return Ok(Box::new(CargoSource::new(path)));
        }

        let maven_path = path.join(MAVEN_FILE);
        if maven_path.exists() {
            return Ok(Box::new(MavenSource::new(path)));
        }
    } else if path.ends_with(CARGO_FILE) {
        return Ok(Box::new(CargoSource::new(path)));
    } else if path.ends_with(MAVEN_FILE) {
        return Ok(Box::new(MavenSource::new(path)));
    }
    bail!(io::ErrorKind::NotFound)
}
