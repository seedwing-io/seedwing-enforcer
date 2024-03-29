use crate::util::result_to_markdown;
use anyhow::{bail, Result};
use clap::{Args, ValueEnum};
use seedwing_enforcer_common::{
    config::Config,
    enforcer::{source::AutoSource, Dependency, Enforcer},
    utils::{pool::Pool, progress::NoProgress},
};
use seedwing_policy_engine::{lang::Severity, runtime::Response};
use serde::Serialize;
use std::env::current_dir;
use std::{fmt::Debug, path::PathBuf};

/// Scan dependencies once
#[derive(Args, Debug)]
#[command(allow_external_subcommands = true)]
pub struct Once {
    /// The root of the project. Defaults to the current directory.
    #[arg(short, long)]
    root: Option<PathBuf>,
    /// The output format
    #[arg(short, long, value_enum, default_value_t = Output::Markdown)]
    output: Output,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Output {
    #[default]
    Markdown,
    Json,
    Yaml,
}

impl Once {
    pub async fn run(self) -> Result<()> {
        let enforcer = self
            .enforcer_setup()
            .await
            .expect("invalid enforcer configuration");

        let dependencies = self.get_deps(enforcer.config.transpose()?).await;

        let result = match dependencies {
            Err(err) => {
                let msg = format!("{:?}", err);
                Outcome {
                    status: AggregatedResult::ConfigError(msg),
                    details: vec![],
                }
            }
            Ok(dependencies) => match enforcer.evaluator.eval(dependencies, NoProgress).await {
                Ok(scan) => {
                    let mut error = false;
                    let mut result = Vec::new();
                    for (dep, outcome) in scan {
                        result.push(PolicyResult::new(dep, &outcome));
                        if outcome.severity == Severity::Error {
                            error = true;
                        }
                    }
                    if error {
                        Outcome {
                            status: AggregatedResult::Rejected,
                            details: result,
                        }
                    } else {
                        Outcome {
                            status: AggregatedResult::Accepted,
                            details: result,
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Error while scanning dependencies : {:?}", e);
                    Outcome {
                        status: AggregatedResult::ConfigError(msg),
                        details: vec![],
                    }
                }
            },
        };

        match self.output {
            Output::Markdown => println!("{}", result_to_markdown(&result)),
            Output::Yaml => println!("{}", serde_yaml::to_string(&result).unwrap()),
            Output::Json => println!("{}", serde_json::to_string(&result).unwrap()),
        }

        match result.status {
            AggregatedResult::Accepted => Ok(()),
            AggregatedResult::ConfigError(msg) => bail!(msg),
            AggregatedResult::Rejected => {
                bail!("")
            }
        }
    }

    async fn get_deps(&self, config: Option<Config>) -> Result<Vec<Dependency>> {
        let path = self.root.clone().unwrap_or(PathBuf::from("./"));
        let source = AutoSource::find_source(path, config).await?;
        source.scan().await
    }

    async fn enforcer_setup(&self) -> Result<Enforcer> {
        let root = match &self.root {
            Some(root) => root.clone(),
            None => current_dir()?,
        };
        let enforcer = Enforcer::new(root, Pool::new()).await;

        let diag = enforcer.diagnostics().await;
        if !diag.is_empty() {
            for (path, issue) in diag {
                println!("{}", path.to_string_lossy());
                for i in issue {
                    println!("\t - {}", i.message)
                }
            }
            bail!("")
        } else {
            Ok(enforcer)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub status: AggregatedResult,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<PolicyResult>,
}

#[derive(Debug, Serialize)]
pub struct PolicyResult {
    pub dependency: Dependency,
    pub response: Response,
}

#[derive(Debug, Serialize)]
pub enum AggregatedResult {
    Accepted,
    ConfigError(String),
    Rejected,
}

impl PolicyResult {
    pub fn new(dependency: Dependency, response: &Response) -> PolicyResult {
        PolicyResult {
            dependency,
            response: response.clone(),
        }
    }
}
