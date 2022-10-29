use crate::{
    adapters::store::Store,
    app_context::AppContext,
    cli::{checkout, commit, context},
};

use super::Checkout;

#[derive(PartialEq, Eq)]
pub enum CheckoutStatus {
    New,
    Existing,
}

/// Used to abstract cli git commands for testings.
pub trait GitCommands {
    /// Get the current git repository name.
    fn get_repo_name(&self) -> anyhow::Result<String>;

    /// Get the current checked out branch name.
    fn get_branch_name(&self) -> anyhow::Result<String>;

    /// Checkout an existing branch of create a new branch if not.
    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()>;

    /// Commit changes and open editor with the template.
    fn commit(&self, msg: &str) -> anyhow::Result<()>;
}

pub trait Commands<C: GitCommands> {
    /// Actions on a context update on the current branch.
    fn current(&self, context: context::Arguments) -> anyhow::Result<()>;

    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, args: checkout::Arguments) -> anyhow::Result<()>;

    /// Actions on a commit.
    fn commit(&self, template: commit::Template) -> anyhow::Result<()>;
}

pub struct CommandActions<'a, C: GitCommands> {
    context: &'a AppContext<C>,
}

impl<'a, C: GitCommands> CommandActions<'a, C> {
    pub fn new(context: &AppContext<C>) -> anyhow::Result<CommandActions<C>> {
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
}

impl<'a, C: GitCommands> Commands<C> for CommandActions<'a, C> {
    fn current(&self, context: context::Arguments) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch_name = self.context.commands.get_branch_name()?;

        let branch = Checkout::new(&branch_name, &repo_name, Some(context.ticket))?;
        Store::insert_or_update(&branch, &self.context.connection)?;

        Ok(())
    }

    fn checkout(&self, checkout: checkout::Arguments) -> anyhow::Result<()> {
        // Attempt to create branch
        let create = self
            .context
            .commands
            .checkout(&checkout.name, CheckoutStatus::New);

        // If the branch already exists check it out
        if create.is_err() {
            self.context
                .commands
                .checkout(&checkout.name, CheckoutStatus::Existing)?;
        }

        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch = Checkout::new(&checkout.name, &repo_name, checkout.ticket.clone())?;
        Store::insert_or_update(&branch, &self.context.connection)?;

        Ok(())
    }

    fn commit(&self, template: commit::Template) -> anyhow::Result<()> {
        let contents = template.commit(self.context)?;
        self.context.commands.commit(&contents)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use anyhow::Context as anyhow_context;
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::app_context::AppContext;

    use super::*;

    #[test]
    fn checkout_success_with_ticket() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = checkout::Arguments {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Store::get(&command.name, &repo, &context.connection)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket.unwrap());

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_with_branch_already_exists_does_not_error() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = checkout::Arguments {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, status| {
                if status == CheckoutStatus::New {
                    Err(anyhow!("branch already exists!"))
                } else {
                    Ok(())
                }
            },
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Store::get(&command.name, &repo, &context.connection)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket.unwrap());

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_on_fail_to_checkout_branch_nothing_is_persisted() -> anyhow::Result<()> {
        // Arrange
        let command = checkout::Arguments {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let repo = Faker.fake::<String>();
        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, _| Err(anyhow!("failed to create or checkout existing branch!")),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        let result = actions.checkout(command.clone());

        // Assert
        assert!(result.is_err());

        let error = Store::get(&command.name, &repo, &context.connection)
            .expect_err("Expected error as there should be no stored branches.");

        assert_eq!(error.to_string(), "Query returned no rows");

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_success_without_ticket_uses_branch_name() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = checkout::Arguments {
            name: Faker.fake::<String>(),
            ticket: None,
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Store::get(&command.name, &repo, &context.connection)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(&branch.ticket, &command.name);

        context.close()?;

        Ok(())
    }

    #[test]
    fn current_success() -> anyhow::Result<()> {
        // Arrange
        let branch_name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let command = context::Arguments {
            ticket: Faker.fake(),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(branch_name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.current(command.clone())?;

        // Assert
        let branch = Store::get(&branch_name, &repo, &context.connection)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket);

        context.close()?;

        Ok(())
    }

    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(dirs)
    }

    fn fake_context<C: GitCommands>(commands: C) -> anyhow::Result<AppContext<C>> {
        let conn = Connection::open_in_memory()?;

        let context = AppContext {
            connection: conn,
            project_dir: fake_project_dir()?,
            commands,
        };

        Ok(context)
    }

    #[derive(Clone)]
    struct GitCommandMock {
        repo: Result<String, String>,
        branch_name: Result<String, String>,
        checkout_res: fn(&str, CheckoutStatus) -> anyhow::Result<()>,
        commit_res: Result<(), String>,
    }

    impl GitCommandMock {
        fn fake() -> GitCommandMock {
            GitCommandMock {
                repo: Ok(Faker.fake()),
                branch_name: Ok(Faker.fake()),
                checkout_res: |_, _| Ok(()),
                commit_res: Ok(()),
            }
        }
    }

    impl GitCommands for GitCommandMock {
        fn get_repo_name(&self) -> anyhow::Result<String> {
            self.repo
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn get_branch_name(&self) -> anyhow::Result<String> {
            self.branch_name
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
            (self.checkout_res)(name, status)
        }

        fn commit(&self, _msg: &str) -> anyhow::Result<()> {
            self.commit_res
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }
    }
}
