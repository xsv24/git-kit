pub mod adapters;
pub mod app_context;
pub mod cli;
pub mod domain;
pub mod utils;

use adapters::Git;
use app_context::AppContext;
use clap::Parser;
use cli::commands::Command;
use domain::commands::{CommandActions, Commands};

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let context = AppContext::new(Git)?;
    let actions = CommandActions::new(&context)?;

    match args.command {
        Command::Commit(template) => {
            actions.commit(template)?;
        }
        Command::Checkout(checkout) => {
            actions.checkout(checkout)?;
        }
        Command::Context(current) => {
            actions.current(current)?;
        }
    };

    context.close()?;

    Ok(())
}
