use crate::domain::{adapters::Git, commands::Commands};

use super::Arguments;

pub fn handler(actions: &dyn Commands, args: Arguments) -> anyhow::Result<()> {
    actions.current(args)?;
    Ok(())
}
