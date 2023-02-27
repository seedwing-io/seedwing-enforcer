use crate::backend::Backend;
use tower_lsp::{LspService, Server};

pub mod backend;
pub mod config;
pub mod enforcer;
pub mod highlight;
pub mod protocol;
pub mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
