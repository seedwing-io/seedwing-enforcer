use crate::cli::Cli;
use clap::Parser;

mod cli;
mod command;

#[tokio::main]
async fn main() {
    Cli::parse().run().await.unwrap();
}
