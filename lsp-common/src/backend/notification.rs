use seedwing_enforcer_common::enforcer::Dependency;
use tower_lsp::lsp_types::notification::Notification;
use url::Url;

macro_rules! lsp {
    ($n:ident[$cmd:literal] -> $p:ident) => {
        pub struct $n;

        impl Notification for $n {
            type Params = $p;
            const METHOD: &'static str = $cmd;
        }
    };
}

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

lsp!(StartOperation["enforcer/startOperation"] -> StartOperationParameters);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartOperationParameters {
    pub token: String,
    pub title: String,
    pub total: usize,
}

lsp!(UpdateOperation["enforcer/updateOperation"] -> UpdateOperationParameters);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateOperationParameters {
    pub token: String,
    pub message: Option<String>,
    pub increment: Option<usize>,
}

lsp!(FinishOperation["enforcer/finishOperation"] -> FinishOperationParameters);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FinishOperationParameters {
    pub token: String,
}
