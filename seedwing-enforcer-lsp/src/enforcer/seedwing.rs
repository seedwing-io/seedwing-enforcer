//! Seedwing enforcer implementation

use crate::config::{self, Config, Dependencies, FILE_NAME_YAML};
use crate::enforcer::{Dependency, Outcome};
use crate::utils::rationale::Rationalizer;
use crate::utils::span_to_range;
use ropey::Rope;
use seedwing_policy_engine::lang::builder::Builder;
use seedwing_policy_engine::runtime::{sources::Ephemeral, BuildError, RuntimeError, World};
use seedwing_policy_engine::value::{self, RuntimeValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::RwLock;
use tokio::task::JoinError;
use tokio_util::task::LocalPoolHandle;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

const DEFAULT_PACKAGE: &str = "enforcer";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Configuration(anyhow::Error),
    #[error("failed to run: {0}")]
    Join(#[from] JoinError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] value::serde::Error),
    #[error("parse policy error: {0}")]
    ParsePolicy(String, Vec<BuildError>),
    #[error("build runtime error")]
    BuildRuntime(Vec<BuildError>),
    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Clone, Debug)]
pub struct Enforcer {
    inner: Arc<RwLock<Inner>>,
}

impl Enforcer {
    pub async fn new(root: impl Into<PathBuf>, pool: LocalPoolHandle) -> Self {
        let mut inner = Inner {
            root: root.into(),
            pool,
            config: None,
        };
        inner.configure().await;
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
    /// Reconfigure the enforcer
    pub async fn configure(&mut self) {
        self.inner.write().await.configure().await;
    }

    /// get current diagnostics for enforcer config itself
    pub async fn diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        self.inner.read().await.diagnostics().await
    }

    /// Evaluate dependencies against the configured enforcer
    pub async fn eval(
        &self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<(Dependency, Outcome)>, Error> {
        self.inner.read().await.eval(dependencies).await
    }
}

#[derive(Debug)]
struct Inner {
    /// Path to the root, containing the `.enforcer` file.
    root: PathBuf,
    pool: LocalPoolHandle,

    config: Option<anyhow::Result<Config>>,
}

impl Inner {
    /// Reconfigure the enforcer
    async fn configure(&mut self) {
        self.config = config::try_load(&self.root).await;
    }

    /// get current diagnostics for enforcer config itself
    async fn diagnostics(&self) -> HashMap<PathBuf, Vec<Diagnostic>> {
        let mut result = HashMap::new();

        // extract config results

        if let Some(Err(err)) = &self.config {
            // failed to load configuration
            result.insert(
                self.root.join(FILE_NAME_YAML),
                vec![Diagnostic {
                    message: err.to_string(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }],
            );
        }

        // eval engine results
        if let Err(err) = self.eval(vec![]).await {
            // as we can't keep the World instance, we also can't prepare and reconfigure it. So
            // we only know that something is wrong when we run the evaluation. However, we need
            // to know as soon as the project is loaded.
            // So we do a dummy run, just to get some details
            match err {
                Error::BuildRuntime(errors) => {
                    if let Some(Ok(Config {
                        dependencies: Some(dependencies),
                    })) = &self.config
                    {
                        let file = self.root.join(&dependencies.policy);
                        let diags = diag_from_build_errors(&file, errors);
                        result.insert(file, diags);
                    }
                }
                Error::ParsePolicy(source, errors) => {
                    let file = self.root.join(&source);
                    let diags = diag_from_build_errors(&file, errors);
                    result.insert(self.root.join(file), diags);
                }
                err => result
                    .entry(self.root.join(FILE_NAME_YAML))
                    .or_default()
                    .push(Diagnostic {
                        message: format!("Failed to initialize engine: {err}"),
                        severity: Some(DiagnosticSeverity::ERROR),
                        ..Default::default()
                    }),
            }
        }

        // return

        result
    }

    pub async fn eval(
        &self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<(Dependency, Outcome)>, Error> {
        // the implementation of this function must keep everything local, as we must provide a
        // function which is `Send`, and seedwing itself is not.

        let config = match &self.config {
            Some(Ok(config)) => config,
            _ => return Ok(all_ok(dependencies)),
        };

        let runner = Runner {
            root: self.root.clone(),
            config: config.clone(),
        };

        self.pool
            .spawn_pinned(move || async move { runner.eval(dependencies).await })
            .await?
    }
}

fn all_ok(dependencies: Vec<Dependency>) -> Vec<(Dependency, Outcome)> {
    dependencies.into_iter().map(|d| (d, Outcome::Ok)).collect()
}

struct Runner {
    root: PathBuf,
    config: Config,
}

impl Runner {
    async fn eval(
        &self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<(Dependency, Outcome)>, Error> {
        let dep_config = match &self.config.dependencies {
            Some(dep_config) => dep_config,
            None => return Ok(all_ok(dependencies)),
        };

        let world = self.build_world(dep_config).await?;

        let mut outcomes = Vec::with_capacity(dependencies.len());

        let requires = format!("{}::{}", DEFAULT_PACKAGE, dep_config.requires);

        for d in dependencies {
            let input: RuntimeValue = d.clone().try_into()?;

            let outcome = world.evaluate(&requires, input, Default::default()).await?;

            let rationale = Rationalizer::new(&outcome).rationale();

            let outcome = match outcome.satisfied() {
                true => Outcome::Ok,
                false => Outcome::Rejected(rationale),
            };

            outcomes.push((d, outcome));
        }

        Ok(outcomes)
    }

    /// Take the configuration and build the world.
    async fn build_world(&self, dep_config: &Dependencies) -> Result<World, Error> {
        let mut builder = Builder::new();

        let file = self.root.join(&dep_config.policy);
        log::info!("Loading from: {}", file.display());

        builder
            .build(Ephemeral::new(DEFAULT_PACKAGE, fs::read_to_string(file)?).iter())
            .map_err(|err| Error::ParsePolicy(dep_config.policy.clone(), err))?;

        let world = builder.finish().await.map_err(Error::BuildRuntime)?;

        Ok(world)
    }
}

fn diag_from_build_errors(file: &Path, errors: Vec<BuildError>) -> Vec<Diagnostic> {
    let file = fs::File::open(file)
        .ok()
        .and_then(|f| Rope::from_reader(f).ok());

    errors
        .into_iter()
        .map(|err| DiagnosticConverter(&file, err).into())
        .collect()
}

pub struct DiagnosticConverter<'a>(pub &'a Option<Rope>, pub BuildError);

impl<'a> From<DiagnosticConverter<'a>> for Diagnostic {
    fn from(value: DiagnosticConverter) -> Self {
        let span = value.1.span();

        let message = match value.1 {
            BuildError::Parser(_, err) => err.to_string(),
            BuildError::ArgumentMismatch(_, _) => "Argument mismatch".into(),
            BuildError::TypeNotFound(_, _, r#type) => format!("Type not found: {type}"),
        };

        let message = format!("{} ({:?})", message, span);

        Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            message,
            range: value
                .0
                .as_ref()
                .and_then(|rope| span_to_range(rope, span))
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}
