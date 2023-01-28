use crate::domain::commands::{Actor, Context};

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    actions.context(Context {
        ticket: args.ticket,
        scope: args.scope,
        link: args.link
    })?;

    Ok(())
}
