use crate::domain::adapters::Git;
use crate::{config::Config, domain::commands::Commands};

use super::Arguments;

pub fn handler(actions: &dyn Commands, config: &Config, args: Arguments) -> anyhow::Result<()> {
    config.validate_template(&args.template)?;
    actions.commit(args)?;

    Ok(())
}
