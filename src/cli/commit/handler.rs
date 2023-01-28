use std::collections::HashMap;

use crate::{
    app_config::{AppConfig, TemplateConfig},
    domain::{commands::{Actor, Commit}, adapters::prompt::{Prompter, SelectItem}},
};

use super::Arguments;

pub fn handler<P: Prompter>(
    actions: &dyn Actor,
    config: &AppConfig,
    args: Arguments,
    prompter: P,
) -> anyhow::Result<()> {
    let template = match args.template {
        Some(template) => config.validate_template(template)?,
        None => prompt_template_select(config.commit.templates.clone(), prompter)?,
    };

    // TODO: Could we do a prompt if no ticket / args found ?
    actions.commit(Commit {
        template,
        ticket: args.ticket,
        message: args.message,
        scope: args.scope,
    })?;

    Ok(())
}

fn prompt_template_select<P: Prompter>(
    templates: HashMap<String, TemplateConfig>,
    prompter: P,
) -> anyhow::Result<String> {
    let items = templates
        .clone()
        .into_iter()
        .map(|(name, template)| SelectItem {
            name,
            description: Some(template.description),
        })
        .collect::<Vec<_>>();

    let selected = prompter.select("Template:", items)?;

    Ok(selected.name)
}
