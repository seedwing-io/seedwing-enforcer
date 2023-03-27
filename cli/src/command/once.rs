use anyhow::{bail, Result};
use clap::{Args, ValueEnum};
use seedwing_enforcer_common::config::Config;
use seedwing_enforcer_common::{
    enforcer::{seedwing::Enforcer, source::AutoSource, Dependency, Outcome},
    utils::{pool::Pool, progress::NoProgress},
};
use serde::Serialize;
use std::{fmt::Debug, path::PathBuf};

#[derive(Args, Debug)]
#[command(about = "Scan dependencies once", allow_external_subcommands = true)]
pub struct Once {
    #[arg(short, long)]
    source: Option<PathBuf>,
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
    #[arg(short, long, value_enum)]
    output: Output,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Output {
    #[default]
    Text,
    Json,
    Yaml,
}

impl Once {
    pub async fn run(self) -> anyhow::Result<()> {
        let enforcer = self
            .enforcer_setup()
            .await
            .expect("invalid enforcer configuration");

        let config = enforcer.get_config().await;

        let dependencies = self.get_deps(config).await;

        let result = if let Err(e) = dependencies {
            let msg = format!("failed scanning dependencies: {:?}", e);
            NamesAreHard {
                status: AggregatedResult::ConfigError(msg),
                details: vec![],
            }
        } else {
            match enforcer.eval(dependencies.unwrap(), NoProgress).await {
                Ok(scan) => {
                    let mut error = false;
                    let mut result = Vec::new();
                    for (dep, outcome) in scan {
                        result.push(PolicyResult::new(dep, &outcome));
                        if outcome.is_failed() {
                            error = true;
                        }
                    }
                    if error {
                        NamesAreHard {
                            status: AggregatedResult::Rejected,
                            details: result,
                        }
                    } else {
                        NamesAreHard {
                            status: AggregatedResult::Accepted,
                            details: result,
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Error while scanning dependencies : {:?}", e);
                    NamesAreHard {
                        status: AggregatedResult::ConfigError(msg),
                        details: vec![],
                    }
                }
            }
        };

        match result.status {
            AggregatedResult::Accepted => {
                match self.output {
                    Output::Text => todo!(),
                    Output::Yaml => println!("{}", serde_yaml::to_string(&result).unwrap()),
                    Output::Json => println!("{}", serde_json::to_string(&result).unwrap()),
                }
                Ok(())
            }
            AggregatedResult::ConfigError(msg) => bail!(msg),
            AggregatedResult::Rejected => {
                match self.output {
                    Output::Text => todo!(),
                    Output::Yaml => println!("{}", serde_yaml::to_string(&result).unwrap()),
                    Output::Json => println!("{}", serde_json::to_string(&result).unwrap()),
                }
                bail!("")
            }
        }
    }

    async fn get_deps(&self, config: Option<Config>) -> Result<Vec<Dependency>> {
        let path = self.source.clone().unwrap_or(PathBuf::from("./"));
        let source = AutoSource::find_source(path, config).await?;
        source.scan().await
    }

    async fn enforcer_setup(&self) -> Result<Enforcer> {
        let mut enforcer = Enforcer::new(dir_path(Some(self.config.clone())), Pool::new()).await;
        enforcer.configure().await;

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

// todo allow providing full path to files and not assume file names
fn dir_path(path: Option<PathBuf>) -> PathBuf {
    let path = path.unwrap_or_else(|| PathBuf::from("./"));

    if path.is_file() {
        path.parent().unwrap().to_path_buf()
    } else {
        path
    }
}

#[derive(Debug, Serialize)]
pub struct NamesAreHard {
    status: AggregatedResult,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: Vec<PolicyResult>,
}

#[derive(Debug, Serialize)]
pub struct PolicyResult {
    dependency: Dependency,
    outcome: Outcome,
    /// Some helpful message if needed ?
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum AggregatedResult {
    Accepted,
    ConfigError(String),
    Rejected,
}

impl PolicyResult {
    pub fn new(dependency: Dependency, outcome: &Outcome) -> PolicyResult {
        PolicyResult {
            dependency,
            outcome: outcome.clone(),
            message: None,
        }
    }
}
