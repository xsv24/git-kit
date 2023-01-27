use crate::{
    cli::{checkout, context},
    domain::models::Branch,
};

use super::CommitArgs;

pub trait Actor {
    /// Actions on a context update on the current branch.
    fn current(&self, args: context::Arguments) -> anyhow::Result<Branch>;

    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, args: checkout::Arguments) -> anyhow::Result<Branch>;

    /// Actions on a commit.
    fn commit(&self, args: CommitArgs) -> anyhow::Result<String>;
}
