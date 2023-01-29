use crate::{domain::commands::Actor, adapters::prompt::Prompt};

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    let context = args.try_into_domain(Prompt)?;
    actions.context(context)?;

    Ok(())
}
