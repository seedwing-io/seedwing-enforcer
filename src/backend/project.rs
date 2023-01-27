use crate::config::FILE_NAME_YAML;
use crate::enforcer::source::maven::MavenSource;
use crate::enforcer::{Enforcer, Error};
use ropey::Rope;
use seedwing_policy_engine::lang::parser::SourceSpan;
use seedwing_policy_engine::runtime::BuildError;
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use tokio_util::task::LocalPoolHandle;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use tower_lsp::Client;
use url::Url;

// FIXME: when we get dropped, send out notifications for the open diagnostics
// TODO: better management of diagnostics
// FIXME: we need to do a lot more caching
// TODO: use SBOM source instead of maven source

/// A project, the real thing we are looking for.
#[derive(Debug)]
pub struct Project {
    client: Client,
    root: PathBuf,
    pool: LocalPoolHandle,
    enforcer: anyhow::Result<Enforcer>,

    /// files which have a policy error reported
    parser_errors: HashSet<Url>,
}

impl Project {
    pub async fn new(client: Client, root: PathBuf, pool: LocalPoolHandle) -> Self {
        let enforcer = Enforcer::new(&root, pool.clone()).await;
        let mut result = Self {
            client,
            root,
            enforcer,
            pool,
            parser_errors: Default::default(),
        };

        result.eval().await;

        result
    }

    /// A file of the project changed
    pub async fn changed(&mut self, path: &Path) {
        log::info!("Project file changed: {}", path.display());
        if path.ends_with(FILE_NAME_YAML) {
            self.reconfigure().await;
        } else if path.ends_with("pom.xml") {
            // source changed
            self.eval().await;
        } else if matches!(path.extension().and_then(|s| s.to_str()), Some("dog")) {
            // policy changed
            self.eval().await;
        }
    }

    async fn reconfigure(&mut self) {
        self.enforcer = Enforcer::new(&self.root, self.pool.clone()).await;
        self.eval().await;
    }

    async fn eval(&mut self) {
        match &self.enforcer {
            Ok(enforcer) => {
                let mut parse_error = None;

                let source = MavenSource::new(&self.root);
                let diag = match enforcer.enforce(source).await {
                    Ok(diags) => diags,
                    Err(Error::ParsePolicy(file, err)) => {
                        // TODO: translate range (of pos) into range of line + char
                        // FIXME: clear error markers
                        // TODO: diff and report new

                        parse_error = Some(file.clone());

                        // FIXME: handle I/O errors
                        let dog_content = Rope::from_reader(File::open(&file).unwrap()).unwrap();

                        let diag = err
                            .into_iter()
                            .map(|e| DiagnosticConverter(&dog_content, e))
                            .map(|e| e.into())
                            .collect();

                        self.client
                            .publish_diagnostics(Url::from_file_path(&file).unwrap(), diag, None)
                            .await;
                        vec![]
                    }
                    Err(err) => {
                        // failed to run enforcer

                        vec![Diagnostic {
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Failed to run enforcer: {err}"),
                            ..Default::default()
                        }]
                    }
                };

                self.clear_parse_errors(parse_error).await;

                // set markers for source
                self.client
                    .publish_diagnostics(self.uri("pom.xml"), diag, None)
                    .await;

                // reset markers for config
                self.client
                    .publish_diagnostics(self.uri(FILE_NAME_YAML), vec![], None)
                    .await;
            }
            Err(err) => {
                // failed to load configuration and set up enforcer

                // set markers for source
                self.client
                    .publish_diagnostics(self.uri("pom.xml"), vec![], None)
                    .await;

                self.client
                    .publish_diagnostics(
                        self.uri(FILE_NAME_YAML),
                        vec![Diagnostic {
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Failed to load configuration: {err}"),
                            ..Default::default()
                        }],
                        None,
                    )
                    .await;
            }
        };
    }

    /// convert a local path, to an absolute file: URL
    fn uri(&self, path: impl AsRef<Path>) -> Url {
        Url::from_file_path(self.root.join(path)).unwrap()
    }

    async fn clear_parse_errors(&mut self, except: Option<String>) {
        let except = except.and_then(|s| Url::from_file_path(s).ok());

        let mut old = self.parser_errors.clone();
        self.parser_errors = HashSet::new();

        if let Some(except) = except {
            old.remove(&except);
            self.parser_errors.insert(except);
        }

        for u in old {
            self.client.publish_diagnostics(u, vec![], None).await;
        }
    }

    pub fn has_marker(path: &Path) -> bool {
        path.join(FILE_NAME_YAML).is_file()
    }
}

pub struct DiagnosticConverter<'a>(pub &'a Rope, pub BuildError);

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
            range: span_to_range(value.0, span).unwrap_or_default(),
            ..Default::default()
        }
    }
}

fn span_to_range(content: &Rope, span: SourceSpan) -> Option<Range> {
    fn convert(content: &Rope, span: SourceSpan) -> Result<Range, ropey::Error> {
        let start_line = content.try_char_to_line(span.start)?;
        let start_pos = span.start - content.try_line_to_char(start_line)?;

        let end_line = content.try_char_to_line(span.end)?;
        let end_pos = span.end - content.try_line_to_char(end_line)?;

        Ok(Range {
            start: Position {
                line: start_line as _,
                character: start_pos as _,
            },
            end: Position {
                line: end_line as _,
                character: end_pos as _,
            },
        })
    }

    convert(content, span).ok()
}
