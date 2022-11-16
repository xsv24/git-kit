use crate::{app_context::AppContext, domain::commands::GitCommands, domain::store::Store};
use clap::Args;

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct Arguments {
    /// Name of the commit template to be used.
    pub template: String,

    /// Issue ticket number related to the commit.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,
}

impl Arguments {
    pub fn commit_message<C: GitCommands, S: Store>(
        &self,
        template: String,
        context: &AppContext<C, S>,
    ) -> anyhow::Result<String> {
        let ticket = self.ticket.as_ref().map(|num| num.trim());

        let ticket_num = match ticket {
            Some(num) => match (num, num.len()) {
                (_, 0) => None,
                (value, _) => Some(value.into()),
            },
            None => context
                .store
                .get(
                    &context.commands.get_branch_name()?,
                    &context.commands.get_repo_name()?,
                )
                .map_or(None, |branch| Some(branch.ticket)),
        };

        let contents = if let Some(ticket) = ticket_num {
            template.replace("{ticket_num}", &format!("[{}]", ticket))
        } else {
            template.replace("{ticket_num}", "").trim().into()
        };

        let contents = match &self.message {
            Some(message) => contents.replace("{message}", message),
            None => contents.replace("{message}", ""),
        };

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        adapters::sqlite::Sqlite,
        config::{CommitConfig, Config, TemplateConfig},
        domain::{
            commands::{CheckoutStatus, GitCommands},
            Branch,
        },
    };
    use anyhow::Context;
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use std::fs;
    use uuid::Uuid;

    use super::*;

    #[derive(Clone)]
    struct TestCommand {
        repo: String,
        branch_name: String,
    }

    impl TestCommand {
        fn fake() -> TestCommand {
            TestCommand {
                repo: Faker.fake(),
                branch_name: Faker.fake(),
            }
        }
    }

    impl GitCommands for TestCommand {
        fn get_repo_name(&self) -> anyhow::Result<String> {
            Ok(self.repo.to_owned())
        }

        fn get_branch_name(&self) -> anyhow::Result<String> {
            Ok(self.branch_name.to_owned())
        }

        fn checkout(&self, _name: &str, _status: CheckoutStatus) -> anyhow::Result<()> {
            todo!()
        }

        fn commit(&self, _msg: &str) -> anyhow::Result<()> {
            todo!()
        }

        fn root_directory(&self) -> anyhow::Result<String> {
            todo!()
        }
    }

    #[test]
    fn empty_ticket_num_removes_square_brackets() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
            config: fake_config(),
        };

        let args = Arguments {
            ticket: Some("".into()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("{}", args.message.unwrap());

        clean(context)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn when_ticket_num_is_empty_square_brackets_are_removed() -> anyhow::Result<()> {
        for ticket in [Some("".into()), Some("   ".into()), None] {
            let context = AppContext {
                store: Sqlite::new(setup_db(None)?)?,
                project_dir: fake_project_dir()?,
                commands: TestCommand::fake(),
                config: fake_config(),
            };

            let args = Arguments {
                template: Faker.fake(),
                ticket,
                message: Some(Faker.fake()),
            };

            let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
            let expected = format!("{}", args.message.unwrap());

            clean(context)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
            config: fake_config(),
        };

        let args = Arguments {
            template: Faker.fake(),
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        clean(context)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
            config: fake_config(),
        };

        let args = Arguments {
            template: Faker.fake(),
            ticket: Some(Faker.fake()),
            message: None,
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] ", args.ticket.unwrap());

        clean(context)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let commands = TestCommand::fake();

        let branch = Branch::new(&commands.branch_name, &commands.repo, None)?;

        let context = AppContext {
            store: Sqlite::new(setup_db(Some(&branch))?)?,
            commands: commands.clone(),
            project_dir: fake_project_dir()?,
            config: fake_config(),
        };

        let args = Arguments {
            ticket: None,
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!(
            "[{}] {}",
            &commands.branch_name,
            args.message.unwrap_or_else(|| "".into())
        );

        clean(context)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    fn fake_args() -> Arguments {
        Arguments {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
        }
    }

    fn get_arguments(args: Option<Arguments>) -> Vec<(&'static str, Arguments)> {
        let args = args.unwrap_or_else(fake_args);

        vec![
            (
                "🐛",
                Arguments {
                    template: "bug".into(),
                    ..args.clone()
                },
            ),
            (
                "✨",
                Arguments {
                    template: "feature".into(),
                    ..args.clone()
                },
            ),
            (
                "🧹",
                Arguments {
                    template: "refactor".into(),
                    ..args.clone()
                },
            ),
            (
                "⚠️",
                Arguments {
                    template: "break".into(),
                    ..args.clone()
                },
            ),
            (
                "📦",
                Arguments {
                    template: "deps".into(),
                    ..args.clone()
                },
            ),
            (
                "📖",
                Arguments {
                    template: "docs".into(),
                    ..args.clone()
                },
            ),
            (
                "🧪",
                Arguments {
                    template: "test".into(),
                    ..args.clone()
                },
            ),
        ]
    }

    #[test]
    fn get_template_config_by_name_key() -> anyhow::Result<()> {
        let config = fake_config();

        for (content, arguments) in get_arguments(None) {
            dbg!(&content, &arguments);
            let template_config = config.get_template_config(&arguments.template)?;
            dbg!(&template_config);
            assert!(template_config.content.contains(content))
        }

        Ok(())
    }
    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .expect("Failed to retrieve 'git-kit' config");

        Ok(dirs)
    }

    fn clean<C: GitCommands, S: Store>(context: AppContext<C, S>) -> anyhow::Result<()> {
        // Some of the directories might not be used so we won't throw it doesn't exist.
        let _ = [
            fs::remove_dir_all(context.project_dir.cache_dir())
                .context("Failed to delete 'cache_dir'"),
            fs::remove_dir(context.project_dir.config_dir())
                .context("Failed to delete 'config_dir'"),
            fs::remove_dir_all(context.project_dir.data_dir())
                .context("Failed to delete 'data_dir'"),
            fs::remove_dir_all(context.project_dir.data_local_dir())
                .context("Failed to delete 'data_local_dir'"),
        ];

        context
            .close()
            .context("Failed to close sqlite connection")?;

        Ok(())
    }

    fn fake_template_config() -> HashMap<String, TemplateConfig> {
        let mut map = HashMap::new();

        map.insert(
            "bug".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} 🐛 {message}".into(),
            },
        );
        map.insert(
            "feature".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} ✨ {message}".into(),
            },
        );
        map.insert(
            "refactor".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} 🧹 {message}".into(),
            },
        );
        map.insert(
            "break".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} ⚠️ {message}".into(),
            },
        );
        map.insert(
            "deps".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} 📦 {message}".into(),
            },
        );
        map.insert(
            "docs".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} 📖 {message}".into(),
            },
        );
        map.insert(
            "test".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "{ticket_num} 🧪 {message}".into(),
            },
        );

        map
    }

    fn fake_config() -> Config {
        Config {
            commit: CommitConfig {
                templates: fake_template_config(),
            },
        }
    }

    fn setup_db(branch: Option<&Branch>) -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            )",
            (),
        )?;

        if let Some(branch) = branch {
            conn.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    branch.created.to_rfc3339(),
                ),
            )?;
        }

        Ok(conn)
    }
}