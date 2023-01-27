use crate::config::{self, Config};
use seedwing_policy_engine::lang::builder::Builder;
use seedwing_policy_engine::runtime::{sources::Ephemeral, BuildError, RuntimeError, World};
use seedwing_policy_engine::value::RuntimeValue;
use std::path::Path;
use std::{fs, io};
use tokio_util::task::LocalPoolHandle;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

mod dependency;
use crate::enforcer::source::Source;
pub use dependency::*;

pub mod source;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse policy: {0} - {1:?}")]
    ParsePolicy(String, Vec<BuildError>),
    #[error("Failed to build runtime: {0:?}")]
    BuildRuntime(Vec<BuildError>),
    #[error("Source error: {0}")]
    Source(anyhow::Error),
    #[error("Evaluation failed")]
    Runtime(#[from] RuntimeError),
    #[error("Serialization issue")]
    Serialization(#[from] seedwing_policy_engine::value::serde::Error),
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug)]
pub struct Enforcer {
    config: Config,
    pool: LocalPoolHandle,
}

struct EnforcerInstance {
    world: World,
    requires: String,
}

impl Enforcer {
    pub async fn new(root: impl AsRef<Path>, pool: LocalPoolHandle) -> anyhow::Result<Self> {
        let config = config::try_load(root.as_ref()).await?.unwrap_or_default();
        Ok(Self { config, pool })
    }

    async fn build_world(config: Config) -> Result<Option<EnforcerInstance>, Error> {
        const DEFAULT_PACKAGE: &str = "enforcer";

        let dep_config = match &config.dependencies {
            Some(dep_config) => dep_config,
            None => return Ok(None),
        };

        let mut builder = Builder::new();

        log::info!("Loading from: {}", &dep_config.policy);

        builder
            .build(Ephemeral::new(DEFAULT_PACKAGE, fs::read_to_string(&dep_config.policy)?).iter())
            .map_err(|err| Error::ParsePolicy(dep_config.policy.clone(), err))?;

        let world = builder.finish().await.map_err(Error::BuildRuntime)?;

        Ok(Some(EnforcerInstance {
            world,
            requires: format!("{}::{}", DEFAULT_PACKAGE, dep_config.requires),
        }))
    }

    pub async fn enforce<S>(&self, source: S) -> Result<Vec<Diagnostic>, Error>
    where
        S: Source + 'static,
    {
        let config = self.config.clone();

        self.pool
            .spawn_pinned(|| async { Self::run_enforce(config, source).await })
            .await
            .map_err(|_| Error::Custom("The enforcer panicked".to_string()))?
    }

    async fn run_enforce<S: Source>(config: Config, source: S) -> Result<Vec<Diagnostic>, Error> {
        let dependencies = source.scan().await.map_err(Error::Source)?;
        log::info!("Dependencies: {dependencies:#?}");
        let dependencies = dependencies
            .into_iter()
            .map(|i| i.try_into())
            .collect::<Result<Vec<RuntimeValue>, _>>()?;

        let instance = match Self::build_world(config).await? {
            Some(instance) => instance,
            None => return Ok(vec![]),
        };

        let outcome = instance
            .world
            .evaluate(&instance.requires, dependencies, Default::default())
            .await?;

        log::info!("Outcome: {outcome:#?}");

        Ok(match outcome.satisfied() {
            true => vec![],
            false => vec![Diagnostic {
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Failed to validate seedwing policy".to_string(),
                ..Default::default()
            }],
        })
    }
}
