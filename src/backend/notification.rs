use crate::enforcer::Dependency;
use tower_lsp::lsp_types::notification::Notification;
use url::Url;

pub struct UpdatedDependencies;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdatedDependenciesParameters {
    pub root: Url,
    pub dependencies: Vec<Dependency>,
}

impl Notification for UpdatedDependencies {
    type Params = UpdatedDependenciesParameters;
    const METHOD: &'static str = "enforcer/updatedDependencies";
}
