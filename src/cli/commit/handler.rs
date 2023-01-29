use crate::{
    app_config::AppConfig,
    domain::{adapters::prompt::Prompter, commands::Actor},
};

use super::Arguments;

pub fn handler<P: Prompter>(
    actions: &dyn Actor,
    config: &AppConfig,
    args: Arguments,
    prompter: P,
) -> anyhow::Result<()> {
    let commit = args.try_into_domain(config, prompter)?;
    actions.commit(commit)?;

    Ok(())
}
