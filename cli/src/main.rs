use crate::cli::Cli;
use clap::Parser;
use std::process::exit;

mod cli;
mod command;

#[tokio::main]
async fn main() {
    match Cli::parse().run().await {
        Ok(_) => exit(0),
        Err(err) => {
            eprintln!("{err}");
            exit(1)
        }
    }
}
