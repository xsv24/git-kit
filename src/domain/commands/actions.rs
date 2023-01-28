use crate::{
    app_context::AppContext,
    domain::{
        adapters::{Git, Store},
        models::Branch,
    }
};

use super::Actor;

pub struct Actions<'a, C: Git, S: Store> {
    context: &'a AppContext<C, S>,
}

impl<'a, C: Git, S: Store> Actions<'a, C, S> {
    pub fn new(context: &AppContext<C, S>) -> Actions<C, S> {
        Actions { context }
    }
}

impl<'a, C: Git, S: Store> Actor for Actions<'a, C, S> {
    fn context(&self, args: super::Context) -> anyhow::Result<Branch> {
        super::context::handler(self.context, args)
    }

    fn checkout(&self, args: super::Checkout) -> anyhow::Result<Branch> {
        super::checkout::handler(self.context, args) 
    }

    fn commit(&self, args: super::Commit) -> anyhow::Result<String> {
        super::commit::handler(self.context, args)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::path::Path;
    use std::path::PathBuf;

    use anyhow::anyhow;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::adapters::sqlite::Sqlite;
    use crate::app_config::AppConfig;
    use crate::app_config::CommitConfig;
    use crate::app_config::TemplateConfig;
    use crate::app_context::AppContext;
    use crate::domain::adapters::CheckoutStatus;
    use crate::domain::adapters::CommitMsgStatus;
    use crate::domain::commands::Checkout;
    use crate::domain::commands::Commit;
    use crate::domain::commands::Context;
    use crate::migrations::{db_migrations, MigrationContext};

    use super::*;

    #[test]
    fn checkout_success_with_ticket() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = Checkout {
            ticket: Some(Faker.fake()),
            ..fake_checkout_args()
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone(), fake_config())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        let expected = Branch {
            name,
            ticket: command.ticket.unwrap(),
            ..branch.clone()
        };

        assert_eq!(branch, expected);

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_with_branch_already_exists_does_not_error() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = fake_checkout_args();

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

        let context = fake_context(git_commands.clone(), fake_config())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        let expected = Branch {
            name,
            ticket: command.ticket.unwrap(),
            link: command.link,
            scope: command.scope,
            created: branch.created,
            data: None,
        };

        assert_eq!(branch, expected);

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_on_fail_to_checkout_branch_nothing_is_persisted() -> anyhow::Result<()> {
        // Arrange
        let command = fake_checkout_args();

        let repo = Faker.fake::<String>();
        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, _| Err(anyhow!("failed to create or checkout existing branch!")),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone(), fake_config())?;
        let actions = Actions::new(&context);

        // Act
        let result = actions.checkout(command.clone());

        // Assert
        assert!(result.is_err());

        let error = context
            .store
            .get_branch(&command.name, &repo)
            .expect_err("Expected error as there should be no stored branches.");

        assert_eq!(error.to_string(), "Query returned no rows");

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_success_without_ticket_uses_branch_name() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = Checkout {
            ticket: None,
            ..fake_checkout_args()
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone(), fake_config())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        let expected = Branch {
            name,
            ticket: command.name,
            scope: command.scope,
            link: command.link,
            data: None,
            created: branch.created,
        };

        assert_eq!(branch, expected);

        context.close()?;

        Ok(())
    }

    #[test]
    fn current_success() -> anyhow::Result<()> {
        // Arrange
        let branch_name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let command = Context {
            ticket: Some(Faker.fake()),
            ..fake_context_args()
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(branch_name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone(), fake_config())?;
        let actions = Actions::new(&context);

        // Act
        actions.context(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&branch_name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        let expected = Branch {
            name,
            ticket: command.ticket.unwrap(),
            link: command.link,
            scope: command.scope,
            ..branch.clone()
        };
        assert_eq!(branch, expected);

        context.close()?;

        Ok(())
    }

    #[test]
    fn commit_message_with_all_arguments_are_injected_into_the_template_with_nothing_persisted(
    ) -> anyhow::Result<()> {
        // Arrange
        let (template, template_config) = fake_template();

        let config = AppConfig {
            commit: CommitConfig {
                templates: HashMap::from([(template.clone(), template_config)]),
            },
        };

        let git_mock = GitCommandMock {
            commit_res: |_, complete| {
                assert_eq!(CommitMsgStatus::Completed, complete);
                Ok(())
            },
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_mock, config)?;
        let actions = Actions::new(&context);

        let args = Commit {
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
            scope: Some(Faker.fake()),
            template,
            ..fake_commit_args()
        };

        // Act
        let contents = actions
            .commit(args.clone())
            .expect("Error performing 'commit' action");

        // Assert
        let expected = format!(
            "[{}] message: '{}', scope: '{}', link: '{}'",
            args.ticket.clone().unwrap(),
            args.message.clone().unwrap(),
            args.scope.unwrap(),
            ""
        );
        assert_eq!(expected, contents);

        context.close()?;

        Ok(())
    }

    #[test]
    fn commit_message_with_no_args_or_stored_branch_defaults_correctly() -> anyhow::Result<()> {
        // Arrange
        let (template, template_config) = fake_template();

        let config = AppConfig {
            commit: CommitConfig {
                templates: HashMap::from([(template.clone(), template_config)]),
            },
        };

        let git_mock = GitCommandMock {
            commit_res: |_, complete| {
                assert_eq!(CommitMsgStatus::InComplete, complete);
                Ok(())
            },
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_mock, config)?;
        let actions = Actions::new(&context);

        let args = Commit {
            ticket: None,
            message: None,
            scope: None,
            template,
        };

        // Act
        let contents = actions
            .commit(args.clone())
            .expect("Error performing 'commit' action");

        // Assert
        assert_eq!("message: '', scope: '', link: ''", contents);

        context.close()?;

        Ok(())
    }

    #[test]
    fn commit_message_with_no_commit_args_defaults_to_stored_branch_values() -> anyhow::Result<()> {
        // Arrange
        let (template, template_config) = fake_template();

        let config = AppConfig {
            commit: CommitConfig {
                templates: HashMap::from([(template.clone(), template_config)]),
            },
        };

        let args = Commit {
            template,
            message: Some(Faker.fake()),
            ticket: None,
            scope: None,
        };

        let context = fake_context(GitCommandMock::fake(), config)?;
        let actions = Actions::new(&context);

        let branch_name = Some(context.git.branch_name()?);
        let repo_name = Some(context.git.repository_name()?);
        let ticket = None;
        let branch = Branch {
            link: Some(Faker.fake()),
            scope: Some(Faker.fake()),
            ..fake_branch(branch_name.clone(), repo_name, ticket)?
        };

        setup_db(&context.store, Some(&branch))?;

        // Act
        let commit_message = actions
            .commit(args.clone())
            .expect("Error performing 'commit' action");

        // Assert
        let expected = format!(
            "[{}] message: '{}', scope: '{}', link: '{}'",
            branch_name.unwrap(),
            args.message.unwrap(),
            branch.scope.unwrap(),
            branch.link.unwrap()
        );

        assert_eq!(expected.trim(), commit_message);

        context.close()?;

        Ok(())
    }

    fn fake_config() -> AppConfig {
        AppConfig {
            commit: CommitConfig {
                templates: fake_template_config(),
            },
        }
    }

    fn fake_context<'a, C: Git>(
        git: C,
        config: AppConfig,
    ) -> anyhow::Result<AppContext<C, Sqlite>> {
        let mut connection = Connection::open_in_memory()?;

        db_migrations(
            &mut connection,
            MigrationContext {
                default_configs: None,
                version: None,
            },
        )?;

        let context = AppContext {
            store: Sqlite::new(connection)?,
            config,
            git,
        };

        Ok(context)
    }

    fn setup_db(store: &Sqlite, branch: Option<&Branch>) -> anyhow::Result<()> {
        if let Some(branch) = branch {
            store.persist_branch(branch.into())?;
        }

        Ok(())
    }

    #[derive(Clone)]
    struct GitCommandMock {
        repo: Result<String, String>,
        branch_name: Result<String, String>,
        checkout_res: fn(&str, CheckoutStatus) -> anyhow::Result<()>,
        commit_res: fn(&Path, CommitMsgStatus) -> anyhow::Result<()>,
        template_file_path: fn() -> anyhow::Result<PathBuf>,
    }

    impl GitCommandMock {
        fn fake() -> GitCommandMock {
            GitCommandMock {
                repo: Ok(Faker.fake()),
                branch_name: Ok(Faker.fake()),
                checkout_res: |_, _| Ok(()),
                commit_res: |_, _| Ok(()),
                template_file_path: || {
                    let temp_file = temp_dir().join(Uuid::new_v4().to_string());
                    Ok(temp_file)
                },
            }
        }
    }

    impl Git for GitCommandMock {
        fn repository_name(&self) -> anyhow::Result<String> {
            self.repo
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn branch_name(&self) -> anyhow::Result<String> {
            self.branch_name
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
            (self.checkout_res)(name, status)
        }

        fn root_directory(&self) -> anyhow::Result<PathBuf> {
            panic!("Did not expect Git 'root_directory' to be called.");
        }

        fn template_file_path(&self) -> anyhow::Result<PathBuf> {
            (self.template_file_path)()
        }

        fn commit_with_template(
            &self,
            template: &Path,
            complete: CommitMsgStatus,
        ) -> anyhow::Result<()> {
            (self.commit_res)(template, complete)
        }
    }

    fn fake_branch(
        name: Option<String>,
        repo: Option<String>,
        ticket: Option<String>,
    ) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());

        Ok(Branch::new(
            &name,
            &repo,
            ticket,
            Faker.fake(),
            Faker.fake(),
        )?)
    }

    fn fake_checkout_args() -> Checkout {
        Checkout {
            name: Faker.fake(),
            ticket: Some(Faker.fake()),
            link: Some(Faker.fake()),
            scope: Some(Faker.fake()),
        }
    }

    fn fake_commit_args() -> Commit {
        Commit {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
            scope: Faker.fake(),
        }
    }

    fn fake_context_args() -> Context {
        Context {
            ticket: Faker.fake(),
            scope: Faker.fake(),
            link: Faker.fake(),
        }
    }

    fn fake_template() -> (String, TemplateConfig) {
        let config = TemplateConfig {
            description: Faker.fake(),
            content: "[{ticket_num}] message: '{message}', scope: '{scope}', link: '{link}'".into(),
        };

        (Faker.fake(), config)
    }

    fn fake_template_config() -> HashMap<String, TemplateConfig> {
        let (_, config) = fake_template();

        HashMap::from([("bug".into(), config)])
    }
}
