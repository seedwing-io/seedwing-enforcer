//! Configuration

use std::{fs, io, path::Path};

pub const FILE_NAME_YAML: &str = ".enforcer.yaml";

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    pub policy: String,
    pub requires: String,
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
/// If the file doesn't exist, we return `None`. Otherwise, we fail.
pub async fn try_load(dir: &Path) -> anyhow::Result<Option<Config>> {
    match fs::File::open(dir.join(FILE_NAME_YAML)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
        Ok(file) => Ok(Some(resolve(serde_yaml::from_reader(&file)?, dir))),
    }
}

/// Find a configuration.
///
/// This function starts to look for a configuration, starting from the provided path, scanning
/// upwards in the tree.
///
/// If a matching file is found, but cannot be read/parsed, it is considered an error.
async fn find(start: impl AsRef<Path>) -> anyhow::Result<Option<Config>> {
    let mut current = Some(start.as_ref());

    while let Some(path) = current {
        if let Some(config) = try_load(path).await? {
            return Ok(Some(config));
        }
        current = path.parent();
    }

    Ok(None)
}
