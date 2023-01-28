use crate::domain::models::Branch;

use super::{Checkout, Commit, Context};

pub trait Actor {
    /// Actions on a context update on the current branch.
    fn context(&self, args: Context) -> anyhow::Result<Branch>;

    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, args: Checkout) -> anyhow::Result<Branch>;

    /// Actions on a commit.
    fn commit(&self, args: Commit) -> anyhow::Result<String>;
}
