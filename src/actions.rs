use crate::{
    branch::Branch,
    cli::{Checkout, Current},
    context::Context,
    git_commands::{CheckoutStatus, GitCommands},
    template::Template,
};

pub trait Actions<C: GitCommands> {
    /// Actions on a context update on the current branch.
    fn current(&self, current: Current) -> anyhow::Result<()>;
    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, checkout: Checkout) -> anyhow::Result<()>;
    /// Actions on a commit.
    fn commit(&self, template: Template) -> anyhow::Result<()>;
}

pub struct CommandActions<C: GitCommands> {
    context: Context<C>,
}

impl<C: GitCommands> CommandActions<C> {
    pub fn new(context: Context<C>) -> anyhow::Result<CommandActions<C>> {
        // TODO: Move into build script ?
        context.connection.execute(
            "CREATE TABLE IF NOT EXISTS branch (
            name TEXT NOT NULL PRIMARY KEY,
            ticket TEXT,
            data BLOB,
            created TEXT NOT NULL
        )",
            (),
        )?;

        Ok(CommandActions { context })
    }

    pub fn close(self) -> anyhow::Result<()> {
        Ok(self.context.close()?)
    }
}

impl<C: GitCommands> Actions<C> for CommandActions<C> {
    fn current(&self, current: Current) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch_name = self.context.commands.get_branch_name()?;

        let branch = Branch::new(&branch_name, &repo_name, Some(current.ticket.clone()))?;
        branch.insert_or_update(&self.context.connection)?;

        Ok(())
    }

    fn checkout(&self, checkout: Checkout) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch = Branch::new(&checkout.name, &repo_name, checkout.ticket.clone())?;
        branch.insert_or_update(&self.context.connection)?;

        // Attempt to create branch
        let create = self
            .context
            .commands
            .checkout(&checkout.name, CheckoutStatus::New);

        // If the branch exists check it out
        if !create.is_err() {
            self.context
                .commands
                .checkout(&checkout.name, CheckoutStatus::Existing)?;
        }

        Ok(())
    }

    fn commit(&self, template: Template) -> anyhow::Result<()> {
        let contents = template.commit(&self.context)?;
        self.context.commands.commit(&contents)?;

        Ok(())
    }
}
