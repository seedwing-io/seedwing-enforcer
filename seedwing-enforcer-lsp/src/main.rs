use seedwing_enforcer_lsp_common::backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
