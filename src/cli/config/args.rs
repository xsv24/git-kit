use crate::domain::{
    adapters::{
        prompt::{Prompter, SelectItem},
        Store,
    },
    commands::config::{AddConfig, Config, SetConfig},
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Add / register a custom config file.
    Add {
        /// Name used to reference the config file.
        name: String,
        /// File path to the config file.
        path: String,
    },
    /// Switch to another config file.
    Set {
        /// Name used to reference the config file.
        name: Option<String>,
    },
    /// Display the current config in use.
    Show,
    /// Reset to the default config.
    Reset,
}

impl Arguments {
    pub fn try_into_domain<S: Store, P: Prompter>(
        &self,
        store: &S,
        prompt: P,
    ) -> anyhow::Result<Config> {
        let config = match self {
            Arguments::Add { name, path } => Config::Add(AddConfig {
                name: name.into(),
                path: path.into(),
            }),
            Arguments::Set { name } => Config::Set(SetConfig {
                name: match name {
                    Some(name) => name.into(),
                    None => prompt_configuration_select(store, prompt)?,
                },
            }),
            Arguments::Show => Config::List,
            Arguments::Reset => Config::Reset,
        };

        Ok(config)
    }
}

fn prompt_configuration_select<S: Store, P: Prompter>(
    store: &S,
    selector: P,
) -> anyhow::Result<String> {
    let configurations: Vec<SelectItem> = store
        .get_configurations()?
        .iter()
        .map(|config| SelectItem {
            name: config.key.clone().into(),
            description: None,
        })
        .collect();

    let selected = selector.select("Configuration:", configurations)?;

    Ok(selected.name)
}
