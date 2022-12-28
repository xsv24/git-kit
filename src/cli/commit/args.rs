use std::fmt::Debug;

use crate::{
    app_context::AppContext,
    domain::adapters::{Git, Store},
    utils::string::{into_option, OptionStr},
};
use anyhow::Context;
use clap::Args;
use regex::Regex;

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

    /// Scope dependency related to the commit.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,
}

impl Arguments {
    fn brackets_regex(target: &str) -> anyhow::Result<Regex> {
        // Replace any surrounding brackets without content with an empty string and remove any trailing spaces.
        // ({target}) | [{target}] | {{target}} | {target}
        // example: http://regexr.com/75aee
        let regex = Regex::new(&format!(
            r"(\(\{{{}\}}\)\s?)|(\[\{{{}\}}\]\s?)|(\{{\{{{}\}}\}}\s?)|(\{{{}\}}\s?)",
            target, target, target, target
        ))?;

        Ok(regex)
    }

    fn replace_or_remove(
        message: String,
        target: &str,
        replace: Option<String>,
    ) -> anyhow::Result<String> {
        let template = format!("{{{}}}", target);

        let message = match &replace.map_empty_to_none() {
            Some(value) => {
                log::info!("replace '{}' from template with '{}'", target, value);
                message.replace(&template, value)
            }
            None => {
                log::info!("removing '{}' from template", target);
                Arguments::brackets_regex(target)
                    .with_context(|| format!("Invalid template for parameter '{}'.", target))?
                    .replace_all(&message, "")
                    .into()
            }
        };

        Ok(message.trim().into())
    }

    pub fn commit_message<C: Git, S: Store>(
        &self,
        template: String,
        context: &AppContext<C, S>,
    ) -> anyhow::Result<String> {
        log::info!("generate commit message for '{}'", &template);
        let ticket = self.ticket.as_ref().map(|num| num.trim());

        let ticket_num = match ticket {
            Some(num) => into_option(num),
            None => context
                .store
                .get_branch(
                    &context.git.get_branch_name()?,
                    &context.git.get_repo_name()?,
                )
                .map_or(None, |branch| Some(branch.ticket)),
        };
        let contents = Self::replace_or_remove(template, "ticket_num", ticket_num)?;
        let contents = Self::replace_or_remove(contents, "message", self.message.clone())?;
        let contents = Self::replace_or_remove(contents, "scope", self.scope.clone())?;

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        adapters::sqlite::Sqlite,
        app_config::{AppConfig, CommitConfig, TemplateConfig},
        domain::{adapters::CheckoutStatus, models::Branch},
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

        fn root_directory(&self) -> anyhow::Result<PathBuf> {
            todo!()
        }
    }

    #[test]
    fn empty_ticket_num_removes_square_brackets() -> anyhow::Result<()> {
        let context = AppContext {
            store: Sqlite::new(setup_db(None)?)?,
            git: TestCommand::fake(),
            config: fake_config(),
        };

        let args = Arguments {
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

        let args = Arguments {
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

            let args = Arguments {
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

        let args = Arguments {
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

        let args = Arguments {
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

        let args = Arguments {
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

        let branch = Branch::new(&commands.branch_name, &commands.repo, None)?;

        let context = AppContext {
            store: Sqlite::new(setup_db(Some(&branch))?)?,
            git: commands.clone(),
            config: fake_config(),
        };

        let args = Arguments {
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

    fn fake_args() -> Arguments {
        Arguments {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
            scope: Faker.fake(),
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
            let template_config = config.get_template_config(&arguments.template)?;
            assert!(template_config.content.contains(content))
        }

        Ok(())
    }

    #[test]
    fn brackets_match() {
        let regex = Arguments::brackets_regex("target").unwrap();
        assert!(regex.is_match("[{target}]"));
        assert!(regex.is_match("[{target}] "));
        assert!(regex.is_match("({target})"));
        assert!(regex.is_match("({target})\t"));
        assert!(regex.is_match("{{target}} "));
        assert!(regex.is_match("{target}"));
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
