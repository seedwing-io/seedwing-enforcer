//! Seedwing enforcer implementation

use crate::enforcer::cache::DefaultCache;
use crate::{
    config::{self, Config, Dependencies, FILE_NAME_YAML},
    enforcer::{cache::Cache, Dependency, Outcome},
    utils::{
        pool::{Pool, PoolError},
        progress::{NoProgress, Progress, ProgressRunner},
        rationale::Rationalizer,
        span_to_range,
    },
};
use lsp_types::{Diagnostic, DiagnosticSeverity};
use ropey::Rope;
use seedwing_policy_engine::{
    lang::builder::Builder,
    runtime::{sources::Ephemeral, BuildError, RuntimeError, World},
    value::{self, RuntimeValue},
};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

const DEFAULT_PACKAGE: &str = "enforcer";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Configuration(anyhow::Error),
    #[error("failed to run: {0}")]
    Join(#[from] PoolError),
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
    pub async fn new(root: impl Into<PathBuf>, pool: Pool) -> Self {
        let mut inner = Inner {
            root: root.into(),
            pool,
            config: None,
            cache: Default::default(),
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
    pub async fn eval<P>(
        &self,
        dependencies: Vec<Dependency>,
        progress: P,
    ) -> Result<Vec<(Dependency, Outcome)>, Error>
    where
        P: Progress + 'static,
    {
        self.inner.read().await.eval(dependencies, progress).await
    }
}

#[derive(Debug)]
struct Inner {
    /// Path to the root, containing the `.enforcer` file.
    root: PathBuf,
    pool: Pool,

    config: Option<anyhow::Result<Config>>,

    cache: DefaultCache,
}

impl Inner {
    /// Reconfigure the enforcer
    async fn configure(&mut self) {
        self.config = config::try_load(&self.root).await;
        self.cache.invalidate();
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
        if let Err(err) = self.eval(vec![], NoProgress).await {
            // as we can't keep the World instance, we also can't prepare and reconfigure it. So
            // we only know that something is wrong when we run the evaluation. However, we need
            // to know as soon as the project is loaded.
            // So we do a dummy run, just to get some details
            match err {
                Error::BuildRuntime(errors) => {
                    if let Some(Ok(Config {
                        dependencies: Some(dependencies),
                        enforcer: _,
                    })) = &self.config
                    {
                        let file = self.root.join(&dependencies.policy);
                        let diags = diag_from_build_errors(&file, errors);
                        result.insert(file, diags);
                    }
                }
                Error::ParsePolicy(source, errors) => {
                    let file = self.root.join(source);
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

    pub async fn eval<P>(
        &self,
        dependencies: Vec<Dependency>,
        progress: P,
    ) -> Result<Vec<(Dependency, Outcome)>, Error>
    where
        P: Progress + 'static,
    {
        // the implementation of this function must keep everything local, as we must provide a
        // function which is `Send`, and seedwing itself is not.

        let config = match &self.config {
            Some(Ok(config)) => config,
            _ => return Ok(all_ok(dependencies)),
        };

        let runner = Runner {
            root: self.root.clone(),
            config: config.clone(),
            progress,
            cache: self.cache.clone(),
        };

        self.pool
            .spawn_pinned(move || async move { runner.eval(dependencies).await })
            .await?
    }
}

fn all_ok(dependencies: Vec<Dependency>) -> Vec<(Dependency, Outcome)> {
    dependencies.into_iter().map(|d| (d, Outcome::Ok)).collect()
}

struct Runner<P: Progress, C: Cache> {
    root: PathBuf,
    config: Config,
    progress: P,
    cache: C,
}

impl<P: Progress, C: Cache> Runner<P, C> {
    async fn eval(
        &self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<(Dependency, Outcome)>, Error> {
        let progress = self
            .progress
            .start("Scanning dependencies", dependencies.len() + 1)
            .await;

        let dep_config = match &self.config.dependencies {
            Some(dep_config) => dep_config,
            None => return Ok(all_ok(dependencies)),
        };

        progress.update(Some("Building world"), None).await;

        let world = self.build_world(dep_config).await?;

        let mut outcomes = Vec::with_capacity(dependencies.len());

        let requires = format!("{}::{}", DEFAULT_PACKAGE, dep_config.requires);

        for d in dependencies {
            progress.update(Some(d.purl.clone()), 1).await;

            match self.cache.get(&d) {
                Some(outcome) => outcomes.push((d, outcome.clone())),
                None => {
                    let input: RuntimeValue = d.clone().try_into()?;
                    let outcome = world.evaluate(&requires, input, Default::default()).await?;
                    let rationale =
                        Rationalizer::new(&outcome).rationale(&self.config.enforcer.rationale);
                    let outcome = match outcome.satisfied() {
                        true => Outcome::Ok,
                        false => Outcome::Rejected(rationale),
                    };

                    self.cache.store(&d, outcome.clone());

                    outcomes.push((d, outcome));
                }
            }
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
            BuildError::PatternNotFound(_, _, r#type) => format!("Pattern not found: {type}"),
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
