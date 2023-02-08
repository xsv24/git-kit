use clap::Args;

use crate::{
    domain::{
        adapters::{
            prompt::{Prompter, SelectItem},
            Store,
        },
        models::{Config, ConfigKey, ConfigStatus},
    },
    entry::Interactive,
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Add / register a custom config file.
    Add(ConfigAdd),
    /// Switch to another config file.
    Set(ConfigSet),
    /// Display the current config in use.
    Show,
    /// Reset to the default config.
    Reset,
}

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct ConfigAdd {
    /// Name used to reference the config file.
    pub name: String,
    /// File path to the config file.
    pub path: String,
}

impl ConfigAdd {
    pub fn try_into_domain(self) -> anyhow::Result<Config> {
        Config::new(self.name.into(), self.path, ConfigStatus::Active)
    }
}

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct ConfigSet {
    /// Name used to reference the config file.
    name: Option<String>,
}

impl ConfigSet {
    pub fn try_into_domain<S: Store, P: Prompter>(
        self,
        store: &S,
        prompt: P,
        interactive: &Interactive,
    ) -> anyhow::Result<ConfigKey> {
        Ok(match self.name {
            Some(name) => name.into(),
            None => prompt_configuration_select(
                store.get_configurations()?,
                prompt,
                interactive.to_owned(),
            )?,
        })
    }
}

fn prompt_configuration_select<P: Prompter>(
    configurations: Vec<Config>,
    selector: P,
    interactive: Interactive,
) -> anyhow::Result<ConfigKey> {
    if interactive == Interactive::Disable {
        anyhow::bail!(clap::Error::raw(
            clap::ErrorKind::MissingRequiredArgument,
            "'name' is required"
        ))
    }

    let configurations: Vec<SelectItem<ConfigKey>> = configurations
        .iter()
        .map(|config| SelectItem {
            name: config.key.clone().into(),
            value: config.key.clone(),
            description: None,
        })
        .collect();

    let selected = selector.select("Configuration:", configurations)?;

    Ok(selected.value)
}
