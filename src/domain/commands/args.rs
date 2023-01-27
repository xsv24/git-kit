use crate::{
    domain::{models::Branch, template::Templator},
    utils::{merge, string::OptionStr},
};

#[derive(Debug, Clone)]
pub struct CommitArgs {
    pub template: String,
    pub ticket: Option<String>,
    pub message: Option<String>,
    pub scope: Option<String>,
}

impl CommitArgs {
    pub fn commit_message(
        &self,
        template: String,
        branch: Option<Branch>,
    ) -> anyhow::Result<String> {
        log::info!("generate commit message for '{}'", &template);
        let (ticket, scope, link) = branch
            .map(|branch| (Some(branch.ticket), branch.scope, branch.link))
            .unwrap_or((None, None, None));

        let ticket = merge(self.ticket.clone().none_if_empty(), ticket.none_if_empty());
        let scope = merge(self.scope.clone().none_if_empty(), scope.none_if_empty());

        let contents = template
            .replace_or_remove("ticket_num", ticket)?
            .replace_or_remove("scope", scope)?
            .replace_or_remove("link", link)?
            .replace_or_remove("message", self.message.clone())?;

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        adapters::sqlite::Sqlite,
        app_config::{AppConfig, CommitConfig, TemplateConfig},
        domain::{
            adapters::{CheckoutStatus, CommitMsgStatus},
            models::Branch,
        },
        migrations::{db_migrations, MigrationContext},
    };
    use fake::{Fake, Faker};
    use rusqlite::Connection;

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

    impl Git for TestCommand {
        fn repository_name(&self) -> anyhow::Result<String> {
            Ok(self.repo.to_owned())
        }

        fn branch_name(&self) -> anyhow::Result<String> {
            Ok(self.branch_name.to_owned())
        }

        fn checkout(&self, _name: &str, _status: CheckoutStatus) -> anyhow::Result<()> {
            panic!("Did not expect Git 'checkout' to be called");
        }

        fn root_directory(&self) -> anyhow::Result<PathBuf> {
            panic!("Did not expect Git 'root_directory' to be called");
        }

        fn template_file_path(&self) -> anyhow::Result<PathBuf> {
            panic!("Did not expect Git 'template_file_path' to be called");
        }

        fn commit_with_template(
            &self,
            _: &std::path::Path,
            _: CommitMsgStatus,
        ) -> anyhow::Result<()> {
            panic!("Did not expect Git 'commit_with_template' to be called");
        }
    }

    #[test]
    fn empty_ticket_num_removes_square_brackets() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = CommitArgs {
            ticket: Some("".into()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("{}", args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn empty_scope_removes_parentheses() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = CommitArgs {
            message: Some(Faker.fake()),
            scope: Some("".into()),
            ticket: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("({scope}) [{ticket_num}] {message}".into(), &context)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn when_ticket_num_is_empty_square_brackets_are_removed() -> anyhow::Result<()> {
        for ticket in [Some("".into()), Some("   ".into()), None] {
            let context = AppContext {
                store: Sqlite::new(setup_db(None)?)?,
                git: TestCommand::fake(),
                config: fake_config(),
            };

            let args = CommitArgs {
                ticket,
                message: Some(Faker.fake()),
                ..fake_args()
            };

            let actual = args.commit_message("[{ticket_num}] {message}".into(), &context)?;
            let expected = format!("{}", args.message.unwrap());

            context.close()?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = CommitArgs {
            template: Faker.fake(),
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("[{ticket_num}] {message}".into(), &context)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = CommitArgs {
            ticket: Some(Faker.fake()),
            message: None,
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("{}", args.ticket.unwrap());

        context.close()?;
        assert_eq!(expected.trim(), actual);

        Ok(())
    }

    #[test]
    fn commit_template_with_empty_brackets_such_as_markdown_checklist_are_not_removed(
    ) -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = CommitArgs {
            message: Some(Faker.fake()),
            ticket: None,
            scope: None,
            ..fake_args()
        };

        let actual = args.commit_message(
            "fix({scope}): [{ticket_num}] {message}\n- done? [ ]".into(),
            &context,
        )?;
        let expected = format!("fix: {}\n- done? [ ]", args.message.unwrap());

        context.close()?;
        assert_eq!(expected.trim(), actual);
        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let commands = TestCommand::fake();

        let branch = Branch::new(&commands.branch_name, &commands.repo, None, None, None)?;

        let context = AppContext {
            store: Sqlite::new(setup_db(Some(&branch))?)?,
            git: commands.clone(),
            config: fake_config(),
        };

        let args = CommitArgs {
            ticket: None,
            ..fake_args()
        };

        let actual = args.commit_message("[{ticket_num}] {message}".into(), &context)?;
        let expected = format!(
            "[{}] {}",
            &commands.branch_name,
            args.message.unwrap_or_else(|| "".into())
        );

        context.close()?;
        assert_eq!(expected.trim(), actual);

        Ok(())
    }

    fn fake_args() -> CommitArgs {
        CommitArgs {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
            scope: Faker.fake(),
        }
    }

    fn get_arguments(args: Option<CommitArgs>) -> Vec<(&'static str, CommitArgs)> {
        let args = args.unwrap_or_else(fake_args);

        vec![
            (
                "🐛",
                CommitArgs {
                    template: "bug".into(),
                    ..args.clone()
                },
            ),
            (
                "✨",
                CommitArgs {
                    template: "feature".into(),
                    ..args.clone()
                },
            ),
            (
                "🧹",
                CommitArgs {
                    template: "refactor".into(),
                    ..args.clone()
                },
            ),
            (
                "⚠️",
                CommitArgs {
                    template: "break".into(),
                    ..args.clone()
                },
            ),
            (
                "📦",
                CommitArgs {
                    template: "deps".into(),
                    ..args.clone()
                },
            ),
            (
                "📖",
                CommitArgs {
                    template: "docs".into(),
                    ..args.clone()
                },
            ),
            (
                "🧪",
                CommitArgs {
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
            let template_config = config.get_template_config(&arguments.template)?;
            assert!(template_config.content.contains(content))
        }

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

    fn fake_config() -> AppConfig {
        AppConfig {
            commit: CommitConfig {
                templates: fake_template_config(),
            },
        }
    }

    fn setup_db(branch: Option<&Branch>) -> anyhow::Result<Connection> {
        let mut conn = Connection::open_in_memory()?;

        db_migrations(
            &mut conn,
            MigrationContext {
                default_configs: None,
                version: None,
            },
        )?;

        if let Some(branch) = branch {
            conn.execute(
                "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    branch.created.to_rfc3339(),
                    &branch.link,
                    &branch.scope
                ),
            )?;
        }

        Ok(conn)
    }
}
