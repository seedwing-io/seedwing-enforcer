use crate::command::lsp::Lsp;
use crate::command::once::Once;
use log::LevelFilter;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Lsp(Lsp),
    Once(Once),
}

#[derive(clap::Parser, Debug)]
#[command(
    author,
    version,
    about = "Seedwing Enforcer",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    /// Be quiet. Conflicts with 'verbose'.
    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue, conflicts_with = "verbose")]
    quiet: bool,
    /// Be more verbose. May be repeated multiple times to increase verbosity.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        let level = match (self.quiet, self.verbose) {
            (true, _) => LevelFilter::Off,
            (_, 0) => LevelFilter::Warn,
            (_, 1) => LevelFilter::Info,
            (_, 2) => LevelFilter::Debug,
            (_, _) => LevelFilter::Trace,
        };

        env_logger::builder().filter_level(level).init();

        match self.command {
            Command::Lsp(command) => command.run().await,
            Command::Once(once) => once.run().await,
        }
    }
}
