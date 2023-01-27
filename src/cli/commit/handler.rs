use std::collections::HashMap;

use crate::{
    app_config::{AppConfig, TemplateConfig},
    cli::select::{SelectItem, SelectorPrompt},
    domain::commands::{Actor, CommitArgs},
};

use super::Arguments;

pub fn handler(
    actions: &dyn Actor,
    config: &AppConfig,
    args: Arguments,
    selector: Box<dyn SelectorPrompt>,
) -> anyhow::Result<()> {
    let template = match args.template {
        Some(template) => config.validate_template(template)?,
        None => prompt_template_select(config.commit.templates.clone(), selector)?,
    };

    // TODO: Could we do a prompt if no ticket / args found ?
    actions.commit(CommitArgs {
        template,
        ticket: args.ticket,
        message: args.message,
        scope: args.scope,
    })?;

    Ok(())
}

fn prompt_template_select(
    templates: HashMap<String, TemplateConfig>,
    selector: Box<dyn SelectorPrompt>,
) -> anyhow::Result<String> {
    let items = templates
        .clone()
        .into_iter()
        .map(|(name, template)| SelectItem {
            name,
            description: Some(template.description),
        })
        .collect::<Vec<_>>();

    let selected = selector.prompt("Template:", items)?;

    Ok(selected.name)
}
