//! Configuration

use std::{fs, io, path::Path};

pub const FILE_NAME_YAML: &str = ".enforcer.yaml";

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
    #[serde(default)]
    pub enforcer: EnforcerConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    pub policy: String,
    pub requires: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnforcerConfig {
    pub source: Option<ManifestType>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RationaleVariant {
    #[default]
    Html,
    Raw,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManifestType {
    Cargo,
    Maven,
}

/// resolve the paths in the configuration
fn resolve(mut config: Config, path: &Path) -> Config {
    if let Some(deps) = &mut config.dependencies {
        deps.policy = path.join(&deps.policy).to_string_lossy().to_string()
    }
    config
}

/// try loading a configuration in a specific path
///
/// If the file doesn't exist, we return `None`. Otherwise, we might fail.
pub async fn try_load(dir: &Path) -> Option<anyhow::Result<Config>> {
    match fs::File::open(dir.join(FILE_NAME_YAML)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => Some(Err(err.into())),
        Ok(file) => Some(
            serde_yaml::from_reader(&file)
                .map_err(|err| err.into())
                .map(|c| resolve(c, dir)),
        ),
    }
}
