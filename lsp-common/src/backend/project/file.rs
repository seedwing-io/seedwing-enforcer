use crate::{
    backend::{
        notification::{UpdatedDependencies, UpdatedDependenciesParameters},
        progress::{run_operation, ClientProgress},
        project::publisher::{Category, DiagnosticPublisher},
    },
    protocol::{commands::SHOW_REPORT, types::Report},
};
use seedwing_enforcer_common::{
    enforcer::{
        seedwing::{self, render::ResponseRenderer, Evaluator},
        source::{
            sbom::{maven::MavenGenerator, SBOM},
            Source,
        },
        Dependency,
    },
    highlight,
};
use seedwing_policy_engine::{
    lang::Severity,
    runtime::{response::Collector, Response},
};
use serde_json::Value;
use std::{collections::HashMap, io, path::PathBuf};
use tower_lsp::{
    lsp_types::{
        CodeActionContext, CodeActionOrCommand, CodeLens, Command, Diagnostic, DiagnosticSeverity,
        Range,
    },
    Client,
};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to gather dependencies: {0}")]
    Source(anyhow::Error),
    #[error("enforcer error: {0}")]
    Enforcer(#[from] seedwing::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// A file in a project which is subject of being verified.
#[derive(Debug)]
pub struct File {
    path: PathBuf,
    enforcer: Evaluator,
    client: Client,

    dependencies: Vec<Dependency>,
    diagnostics: HashMap<Url, Vec<Diagnostic>>,
}

impl File {
    pub fn new(path: PathBuf, client: Client, enforcer: Evaluator) -> Self {
        Self {
            path,
            enforcer,
            client,
            dependencies: Default::default(),
            diagnostics: Default::default(),
        }
    }

    /// build the project, which in this case means to gather and validate dependencies
    pub async fn build(&mut self, publisher: &mut DiagnosticPublisher) {
        let root = match Url::from_file_path(&self.path) {
            Ok(url) => url,
            Err(_) => return,
        };

        match self.process().await {
            Ok(()) => {}
            Err(err) => {
                log::warn!("Failed to run: {err}");
                self.dependencies.clear();
                self.diagnostics = HashMap::from([(
                    root.clone(),
                    vec![Diagnostic {
                        message: format!("Failed to run enforcer: {err}"),
                        severity: Some(DiagnosticSeverity::ERROR),
                        ..Default::default()
                    }],
                )]);
            }
        }

        log::info!("Returned from operation");

        // publish outcome
        publisher
            .publish(Category::Source, self.diagnostics.clone())
            .await;

        log::info!("Send dependency update");

        self.client
            .send_notification::<UpdatedDependencies>(UpdatedDependenciesParameters {
                root,
                dependencies: self.dependencies.clone(),
            })
            .await;
    }

    async fn process(&mut self) -> Result<(), Error> {
        let root = match self.path.parent() {
            Some(parent) => parent,
            None => return Ok(()),
        };

        // refresh dependencies
        let source = SBOM::new(MavenGenerator::new(root));
        self.dependencies = run_operation(
            self.client.clone(),
            "Gathering dependencies",
            1,
            |_progress| async { source.scan().await.map_err(Error::Source) },
        )
        .await?;

        // evaluate policies

        let response = self
            .enforcer
            .eval(
                self.dependencies.clone(),
                ClientProgress(self.client.clone()),
            )
            .await?;

        // render diagnostics

        let mut diags = HashMap::<Url, Vec<Diagnostic>>::new();

        for (dependency, response) in response {
            match response.severity {
                Severity::None => {
                    // ignore succeeded entries
                }
                severity => {
                    if let Ok((url, range)) = source.highlight(&dependency) {
                        diags.entry(url).or_default().push({
                            let collected = Collector::new(&response).highest_severity().collect();
                            let message = collected
                                .iter()
                                .map(|r| r.reason.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");

                            Diagnostic {
                                severity: match severity {
                                    Severity::None => None,
                                    Severity::Advice => Some(DiagnosticSeverity::INFORMATION),
                                    Severity::Warning => Some(DiagnosticSeverity::WARNING),
                                    Severity::Error => Some(DiagnosticSeverity::ERROR),
                                },
                                message: format!("{}: {}", dependency.purl, message),
                                range: range.into(),
                                data: Self::make_data(&dependency, &collected, &response).ok(),
                                ..Default::default()
                            }
                        });
                    }
                }
            }
        }

        self.diagnostics = diags;

        Ok(())
    }

    fn make_data(
        dependency: &Dependency,
        collected: &[Response],
        original: &Response,
    ) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(&Report {
            title: dependency.purl.to_string(),
            html: format!(
                r#"
<div class="swe-response">
    <div class="swe-reasons">
        {response}
    </div>
    <div class="swe-response-raw">
        <details>
            <summary>Raw JSON</summary>
            <code><pre>{raw}</pre></code>
        </details>
    </div>
</div>
"#,
                response = ResponseRenderer(collected).render(),
                raw = serde_json::to_string_pretty(original).unwrap_or_default()
            ),
        })?)
    }

    pub async fn code_lens(&self) -> anyhow::Result<Vec<CodeLens>> {
        let root = match Url::from_file_path(&self.path) {
            Ok(url) => url,
            Err(_) => return Ok(vec![]),
        };

        if let Some(diags) = self.diagnostics.get(&root) {
            self.collect_code_lens(diags)
        } else {
            Ok(vec![])
        }
    }

    fn collect_code_lens(&self, diags: &[Diagnostic]) -> anyhow::Result<Vec<CodeLens>> {
        let mut reports: HashMap<highlight::Range, Vec<Value>> = HashMap::new();

        for d in diags {
            let range = highlight::Range::from(d.range);

            let entry = reports.entry(range).or_default();
            if let Some(data) = &d.data {
                entry.push(data.clone());
            }
        }

        let mut result = vec![];

        for (k, reports) in reports {
            result.push(CodeLens {
                range: k.into(),
                command: Some(Self::create_report_command(reports)?),
                data: None,
            });
        }

        Ok(result)
    }

    pub async fn code_action(
        &self,
        _range: &Range,
        _context: &CodeActionContext,
    ) -> anyhow::Result<Vec<CodeActionOrCommand>> {
        Ok(vec![])

        /*
        log::info!("Code actions for - range: {range:?}, contex: {context:?}");

        let root = match Url::from_file_path(&self.path) {
            Ok(url) => url,
            Err(_) => return Ok(vec![]),
        };

        let diags = if let Some(diags) = self.diagnostics.get(&root) {
            diags
        } else {
            return Ok(vec![]);
        };

        let req_range = highlight::Range::from(*range);

        // gather reports

        let mut report: Vec<Value> = vec![];

        for d in diags {
            let range = highlight::Range::from(d.range);
            if range.contains(&req_range.start) {
                if let Some(data) = &d.data {
                    report.push(data.clone());
                }
            }
        }

        // provide code action

        Ok(if report.is_empty() {
            vec![]
        } else {
            vec![CodeActionOrCommand::Command(Self::create_report_command(
                report,
            )?)]
        })*/
    }

    fn create_report_command(report: Vec<Value>) -> anyhow::Result<Command> {
        Ok(Command {
            title: format!("Show Report ({} entries)", report.len()),
            command: SHOW_REPORT.to_string(),
            arguments: Some(vec![serde_json::to_value(&report)?]),
        })
    }
}
