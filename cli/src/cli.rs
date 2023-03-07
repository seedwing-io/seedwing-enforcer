use crate::command::lsp::Lsp;
use crate::command::once::Once;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Lsp(Lsp),
    Once(Once),
}

#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about="Seedwing Enforcer",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        env_logger::init();

        match self.command {
            Command::Lsp(command) => command.run().await,
            Command::Once(once) => once.run().await,
        }
    }
}
