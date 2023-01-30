use clap::Parser;
use git_kit::{entry::Cli, adapters::prompt::Prompt};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut context = cli.init()?;
    let result = cli.commands.execute(&mut context, Prompt);

    // close the connection no matter if we error or not.
    context.close()?;

    result
}
