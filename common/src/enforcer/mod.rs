mod dependency;

use crate::config::{try_load, Config};
use crate::enforcer::seedwing::Evaluator;
use crate::utils::pool::Pool;
pub use dependency::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod cache;
pub mod seedwing;
pub mod source;

/// An enforcer project
#[derive(Debug)]
pub struct Enforcer {
    root: PathBuf,
    #[deprecated(note = "The enforcer should take care of this")]
    pub evaluator: Evaluator,
    pub config: Option<anyhow::Result<Config>>,
}

impl Enforcer {
    pub async fn new(root: impl Into<PathBuf>, pool: Pool) -> Self {
        let root = root.into();
        let evaluator = Evaluator::new(&root, pool).await;
        let config = try_load(&root).await;

        #[allow(deprecated)]
        Self {
            root,
            evaluator,
            config,
        }
    }

    pub async fn configure(&mut self) {
        #[allow(deprecated)]
        self.evaluator.configure().await;
    }

    pub async fn diagnostics(&self) -> HashMap<PathBuf, Vec<lsp_types::Diagnostic>> {
        #[allow(deprecated)]
        self.evaluator.diagnostics().await
    }
}
