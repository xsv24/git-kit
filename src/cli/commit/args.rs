use std::{collections::HashMap, fmt::Debug};

use clap::Args;

use crate::{
    domain::{
        adapters::prompt::{Prompter, SelectItem},
        commands::commit::Commit,
    },
    entry::Interactive,
    template_config::{Template, TemplateConfig},
};

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct Arguments {
    /// Name of the commit template to be used.
    pub template: Option<String>,

    /// Issue ticket number related to the commit.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(
        &self,
        config: &TemplateConfig,
        prompter: P,
        interactive: &Interactive,
    ) -> anyhow::Result<Commit> {
        let template = match &self.template {
            Some(template) => template.into(),
            None => Self::prompt_template_select(
                config.commit.templates.clone(),
                prompter,
                interactive.to_owned(),
            )?,
        };

        // TODO: Could we do a prompt if no ticket / args found ?
        Ok(Commit {
            template: config.get_template_config(&template)?.clone(),
            ticket: self.ticket.clone(),
            message: self.message.clone(),
            scope: self.scope.clone(),
        })
    }

    fn prompt_template_select<P: Prompter>(
        templates: HashMap<String, Template>,
        prompter: P,
        interactive: Interactive,
    ) -> anyhow::Result<String> {
        if interactive == Interactive::Disable {
            anyhow::bail!(clap::Error::raw(
                clap::ErrorKind::MissingRequiredArgument,
                "'template' is required"
            ))
        }

        let items = templates
            .into_iter()
            .map(|(name, template)| SelectItem {
                name: name.clone(),
                value: name,
                description: Some(template.description),
            })
            .collect::<Vec<_>>();

        let selected = prompter.select("Template:", items)?;

        Ok(selected.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use fake::{Fake, Faker};

    use crate::{
        domain::{adapters::prompt::SelectItem, errors::UserInputError},
        template_config::CommitConfig,
    };

    #[test]
    fn try_into_domain_with_no_interactive_prompts() -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: Some(key.clone()),
            ..fake_args()
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(&config, prompt, &Interactive::Disable)?;

        let expected = Commit {
            template: value,
            ticket: args.ticket.clone(),
            scope: args.scope.clone(),
            message: args.message.clone(),
        };

        assert_eq!(expected.template.content, actual.template.content);
        assert_eq!(expected.message, actual.message);
        assert_eq!(expected.scope, actual.scope);
        assert_eq!(expected.ticket, actual.ticket);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_used_if_none() -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: None,
            ticket: None,
            scope: None,
            message: None,
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let text_prompt = Faker.fake::<Option<String>>();

        let prompt = PromptTest {
            select_item_name: Ok(key.clone()),
            text_result: Ok(text_prompt.clone()),
        };

        let actual = args
            .clone()
            .try_into_domain(&config, prompt, &Interactive::Enable)?;

        let expected = Commit {
            template: value,
            ticket: text_prompt.clone(),
            scope: text_prompt.clone(),
            message: text_prompt.clone(),
        };

        assert_eq!(expected.template.description, actual.template.description);
        assert_eq!(args.message, actual.message);
        assert_eq!(args.scope, actual.scope);
        assert_eq!(args.ticket, actual.ticket);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_not_used_if_value_is_already_provided(
    ) -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: Some(key.clone()),
            ticket: Some(Faker.fake()),
            scope: Some(Faker.fake()),
            message: Some(Faker.fake()),
            ..fake_args()
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(&config, prompt, &Interactive::Enable)?;

        let expected = Commit {
            template: value,
            ticket: args.ticket.clone(),
            scope: args.scope.clone(),
            message: args.message.clone(),
        };

        assert_eq!(expected.template.description, actual.template.description);
        assert_eq!(expected.message, actual.message);
        assert_eq!(expected.scope, actual.scope);
        assert_eq!(expected.ticket, actual.ticket);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_disabled_and_no_template_provided_an_error_is_thrown(
    ) {
        let args = Arguments {
            template: None,
            ..fake_args()
        };

        let config = fake_template_config(None);

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let error = args
            .clone()
            .try_into_domain(&config, prompt, &Interactive::Disable)
            .unwrap_err();

        assert_eq!(error.to_string(), "error: 'template' is required");
    }

    pub struct PromptTest {
        select_item_name: anyhow::Result<String>,
        text_result: anyhow::Result<Option<String>>,
    }

    impl Prompter for PromptTest {
        fn text(&self, name: &str, _: Option<String>) -> Result<Option<String>, UserInputError> {
            match &self.text_result {
                Ok(option) => Ok(option.clone()),
                Err(_) => Err(UserInputError::Validation { name: name.into() }),
            }
        }

        fn select<T>(
            &self,
            name: &str,
            options: Vec<SelectItem<T>>,
        ) -> Result<SelectItem<T>, UserInputError> {
            match &self.select_item_name {
                Ok(name) => options
                    .into_iter()
                    .find(|i| i.name == name.clone())
                    .context("Failed to get item")
                    .map_err(|_| UserInputError::Validation { name: name.into() }),
                Err(_) => Err(UserInputError::Validation { name: name.into() }),
            }
        }
    }

    fn fake_template(description: &str) -> Template {
        Template {
            description: description.into(),
            content: Faker.fake(),
        }
    }

    fn fake_template_config(selected: Option<(String, Template)>) -> TemplateConfig {
        let mut map = HashMap::from([
            ("option-1".into(), fake_template("option-1")),
            ("option-2".into(), fake_template("option-2")),
            ("option-3".into(), fake_template("option-3")),
        ]);

        if let Some((key, item)) = selected {
            map.insert(key, item);
        }

        let config = CommitConfig { templates: map };

        TemplateConfig { commit: config }
    }

    fn fake_args() -> Arguments {
        Arguments {
            template: Faker.fake(),
            ticket: Faker.fake(),
            scope: Faker.fake(),
            message: Faker.fake(),
        }
    }
}
