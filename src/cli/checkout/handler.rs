use crate::{domain::commands::{Actor, Checkout}, adapters::prompt::Prompt};

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    let checkout = args.try_into_domain(Prompt)?;
    actions.checkout(checkout)?;

    Ok(())
}
