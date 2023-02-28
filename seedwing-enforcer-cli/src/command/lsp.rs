use seedwing_enforcer_lsp_common::backend::Backend;
use tower_lsp::{LspService, Server};

#[derive(clap::Args, Debug)]
#[command(about = "Language Server Protocol", allow_external_subcommands = true)]
pub struct Lsp {}

impl Lsp {
    pub async fn run(self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(Backend::new).finish();
        Server::new(stdin, stdout, socket).serve(service).await;

        Ok(())
    }
}
