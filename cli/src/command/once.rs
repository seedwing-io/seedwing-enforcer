use clap::{Args, ValueEnum};
use seedwing_enforcer_common::{
    enforcer::{
        seedwing::Enforcer,
        source::{maven::MavenSource, Source},
        Dependency, Outcome,
    },
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
        let res = self.inner_run().await;

        match self.output {
            Output::Text => todo!(),
            Output::Yaml => println!("{}", serde_yaml::to_string(&res).unwrap()),
            Output::Json => println!("{}", serde_json::to_string(&res).unwrap()),
        }

        match res.status {
            AggregatedResult::Accepted => Ok(()),
            AggregatedResult::ConfigError(msg) => anyhow::bail!(msg),
            AggregatedResult::Rejected => anyhow::bail!(""),
        }
    }

    async fn inner_run(&self) -> NamesAreHard {
        let pom = MavenSource::new(dir_path(self.source.clone()));
        let dependencies = pom.scan().await;
        if let Err(e) = dependencies {
            let msg = format!("failed scanning dependencies: {:?}", e);
            return NamesAreHard {
                status: AggregatedResult::ConfigError(msg),
                details: vec![],
            };
        }

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
            let msg = "invalid enforcer configuration.".to_string();
            return NamesAreHard {
                status: AggregatedResult::ConfigError(msg),
                details: vec![],
            };
        }

        return match enforcer.eval(dependencies.unwrap(), NoProgress).await {
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
        };
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
        let message = match outcome {
            Outcome::Ok => None,
            Outcome::Rejected(msg) => Some(msg.clone()),
        };
        PolicyResult {
            dependency,
            outcome: outcome.clone(),
            message,
        }
    }
}
