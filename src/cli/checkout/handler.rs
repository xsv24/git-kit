use crate::domain::commands::{Actor, Checkout};

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    actions.checkout(Checkout {
        name: args.name,
        ticket: args.ticket,
        scope: args.scope,
        link: args.link
    })?;

    Ok(())
}
