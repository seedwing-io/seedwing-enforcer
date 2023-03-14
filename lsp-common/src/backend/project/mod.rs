use crate::backend::project::publisher::{Category, DiagnosticPublisher};
use seedwing_enforcer_common::{
    config::FILE_NAME_YAML, enforcer::seedwing::Enforcer, utils::pool::Pool,
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tower_lsp::{
    lsp_types::{CodeActionContext, CodeActionOrCommand, CodeLens, Range},
    Client,
};

mod file;
mod publisher;

// FIXME: when we get dropped, send out notifications for the open diagnostics
// TODO: better management of diagnostics
// FIXME: we need to do a lot more caching

/// A project, the real thing we are looking for.
#[derive(Debug)]
pub struct Project {
    client: Client,
    root: PathBuf,
    enforcer: Enforcer,

    /// publisher for diagnostic information
    publisher: DiagnosticPublisher,

    /// File which we track for enforcing (not the configuration)
    files: HashMap<PathBuf, file::File>,
}

impl Project {
    pub async fn new(client: Client, root: PathBuf, pool: Pool) -> Self {
        let enforcer = Enforcer::new(&root, pool).await;
        let publisher = DiagnosticPublisher::new(client.clone());
        let mut result = Self {
            client,
            root,
            enforcer,
            publisher,
            files: Default::default(),
        };

        result.initial_scan().await;
        result.reconfigure().await;

        result
    }

    /// Perform an initial scan
    async fn initial_scan(&mut self) {
        let pom = self.root.join("pom.xml");
        if pom.is_file() {
            let file = file::File::new(pom.clone(), self.client.clone(), self.enforcer.clone());
            log::info!("Initially adding: {}", pom.display());
            self.files.insert(pom, file);
        }
    }

    /// A file of the project changed
    pub async fn changed(&mut self, path: &Path) {
        log::info!("Project file changed: {}", path.display());

        if let Some(file) = self.files.get_mut(path) {
            // content changed
            file.build(&mut self.publisher).await;
        } else if path.ends_with("pom.xml") {
            log::info!("Adding: {}", path.display());
            // FIXME: don't descend into sub-dirs, only root level pom
            let mut file = file::File::new(path.into(), self.client.clone(), self.enforcer.clone());
            file.build(&mut self.publisher).await;
            self.files.insert(path.to_path_buf(), file);
        } else if path.ends_with(FILE_NAME_YAML) {
            // configuration changed
            self.reconfigure().await;
        } else if matches!(path.extension().and_then(OsStr::to_str), Some("dog")) {
            // policy changed
            // TODO: we reconfigure on any .dog change, we could limit this to the ones used
            self.reconfigure().await;
        }
    }

    async fn reconfigure(&mut self) {
        self.enforcer.configure().await;

        let diags = self.enforcer.diagnostics().await;
        self.publisher.publish_file(Category::Enforcer, diags).await;

        // now re-evaluate all
        for file in &mut self.files.values_mut() {
            file.build(&mut self.publisher).await;
        }
    }

    pub fn has_marker(path: &Path) -> bool {
        path.join(FILE_NAME_YAML).is_file()
    }

    pub async fn code_lens(&self, path: &Path) -> anyhow::Result<Vec<CodeLens>> {
        if let Some(file) = self.files.get(path) {
            file.code_lens().await
        } else {
            Ok(vec![])
        }
    }

    pub async fn code_action(
        &self,
        path: &Path,
        range: &Range,
        context: &CodeActionContext,
    ) -> anyhow::Result<Vec<CodeActionOrCommand>> {
        if let Some(file) = self.files.get(path) {
            file.code_action(range, context).await
        } else {
            Ok(vec![])
        }
    }
}
