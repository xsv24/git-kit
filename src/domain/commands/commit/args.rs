use crate::{domain::{template::Templator, models::Branch}, utils::{string::OptionStr, merge}};

#[derive(Debug, Clone)]
pub struct Commit {
    pub template: String,
    pub ticket: Option<String>,
    pub message: Option<String>,
    pub scope: Option<String>,
}

impl Commit {
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
    use fake::{Faker, Fake};

    use super::*;
    use crate::{
        app_config::{AppConfig, CommitConfig, TemplateConfig},
        domain::models::Branch,
    };
    use std::collections::HashMap;

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

    #[test]
    fn empty_ticket_num_removes_square_brackets() -> anyhow::Result<()> {
        let args = Commit {
            ticket: Some("".into()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), None)?;
        let expected = format!("{}", args.message.unwrap());

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn empty_scope_removes_parentheses() -> anyhow::Result<()> {
        let args = Commit {
            message: Some(Faker.fake()),
            scope: Some("".into()),
            ticket: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("({scope}) [{ticket_num}] {message}".into(), None)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn when_ticket_num_is_empty_square_brackets_are_removed() -> anyhow::Result<()> {
        for ticket in [Some("".into()), Some("   ".into()), None] {
            let args = Commit {
                ticket,
                message: Some(Faker.fake()),
                ..fake_args()
            };

            let actual = args.commit_message("[{ticket_num}] {message}".into(), None)?;
            let expected = format!("{}", args.message.unwrap());

            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let args = Commit {
            template: Faker.fake(),
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let actual = args.commit_message("[{ticket_num}] {message}".into(), None)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let args = Commit {
            ticket: Some(Faker.fake()),
            message: None,
            ..fake_args()
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), None)?;
        let expected = format!("{}", args.ticket.unwrap());

        assert_eq!(expected.trim(), actual);

        Ok(())
    }

    #[test]
    fn commit_template_with_empty_brackets_such_as_markdown_checklist_are_not_removed(
    ) -> anyhow::Result<()> {
        let args = Commit {
            message: Some(Faker.fake()),
            ticket: None,
            scope: None,
            ..fake_args()
        };

        let actual = args.commit_message(
            "fix({scope}): [{ticket_num}] {message}\n- done? [ ]".into(),
            None,
        )?;
        let expected = format!("fix: {}\n- done? [ ]", args.message.unwrap());

        assert_eq!(expected.trim(), actual);
        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let commands = TestCommand::fake();

        let branch = Branch::new(&commands.branch_name, &commands.repo, None, None, None)?;

        let args = Commit {
            ticket: None,
            ..fake_args()
        };

        let actual = args.commit_message("[{ticket_num}] {message}".into(), Some(branch))?;
        let expected = format!(
            "[{}] {}",
            &commands.branch_name,
            args.message.unwrap_or_else(|| "".into())
        );

        assert_eq!(expected.trim(), actual);

        Ok(())
    }

    fn fake_args() -> Commit {
        Commit {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
            scope: Faker.fake(),
        }
    }

    fn get_arguments(args: Option<Commit>) -> Vec<(&'static str, Commit)> {
        let args = args.unwrap_or_else(fake_args);

        vec![
            (
                "🐛",
                Commit {
                    template: "bug".into(),
                    ..args.clone()
                },
            ),
            (
                "✨",
                Commit {
                    template: "feature".into(),
                    ..args.clone()
                },
            ),
            (
                "🧹",
                Commit {
                    template: "refactor".into(),
                    ..args.clone()
                },
            ),
            (
                "⚠️",
                Commit {
                    template: "break".into(),
                    ..args.clone()
                },
            ),
            (
                "📦",
                Commit {
                    template: "deps".into(),
                    ..args.clone()
                },
            ),
            (
                "📖",
                Commit {
                    template: "docs".into(),
                    ..args.clone()
                },
            ),
            (
                "🧪",
                Commit {
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

}
