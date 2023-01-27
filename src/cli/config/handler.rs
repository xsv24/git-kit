use crate::AppConfig;
use crate::{
    cli::select::{SelectItem, SelectorPrompt},
    domain::{
        adapters::{Git, Store},
        models::{Config, ConfigKey, ConfigStatus},
    },
};

use super::Arguments;
use colored::Colorize;

pub fn handler<S: Store, G: Git>(
    store: &mut S,
    git: &G,
    arguments: Arguments,
    selector: Box<dyn SelectorPrompt>,
) -> anyhow::Result<()> {
    local_config_warning(git)?;

    match arguments {
        Arguments::Add { name, path } => {
            let config = add(store, name, path)?;
            println!("🟢 {}", config.key.to_string().green());
        }
        Arguments::Set { name } => {
            let config = set(store, name, selector)?;
            println!("🟢 {} (Active) ", config.key.to_string().green());
        }
        Arguments::Reset => {
            let config = reset(store)?;
            println!("🟢 Config reset to {}", config.key.to_string().green());
        }
        Arguments::Show => {
            let configurations = list(store)?;

            for config in configurations {
                let key = config.key.to_string();
                let path = config.path.display();

                match config.status {
                    ConfigStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
                    ConfigStatus::Disabled => println!("🔴 {} ➜ '{}'", key, path),
                }
            }
        }
    };

    Ok(())
}

fn add<S: Store>(store: &mut S, name: String, path: String) -> anyhow::Result<Config> {
    let config = Config::new(name.into(), path, ConfigStatus::Active)?;

    store.persist_config(&config)?;

    store.set_active_config(config.key)
}

fn prompt_configuration_select<S: Store>(
    store: &mut S,
    selector: Box<dyn SelectorPrompt>,
) -> anyhow::Result<String> {
    let configurations: Vec<SelectItem> = store
        .get_configurations()?
        .iter()
        .map(|config| SelectItem {
            name: config.key.clone().into(),
            description: None,
        })
        .collect();

    let selected = selector.prompt("Configuration:", configurations)?;

    Ok(selected.name)
}

fn set<S: Store>(
    store: &mut S,
    name: Option<String>,
    selector: Box<dyn SelectorPrompt>,
) -> anyhow::Result<Config> {
    let name = match name {
        Some(name) => name,
        None => prompt_configuration_select(store, selector)?
    };

    store.set_active_config(ConfigKey::from(name))
}

fn reset<S: Store>(store: &mut S) -> anyhow::Result<Config> {
    store.set_active_config(ConfigKey::Default)
}

fn list<S: Store>(store: &mut S) -> anyhow::Result<Vec<Config>> {
    let mut configurations = store.get_configurations()?;
    configurations.sort_by_key(|c| c.status.clone());

    Ok(configurations)
}

fn local_config_warning<G: Git>(git: &G) -> anyhow::Result<()> {
    let local_config_path = AppConfig::join_config_filename(&git.root_directory()?);

    if local_config_path.exists() {
        println!("{}: 'Active' configurations are currently overridden due to a local repo configuration being used.\n", "⚠️ Warning".yellow());
    }

    Ok(())
}
