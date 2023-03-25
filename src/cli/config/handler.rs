use crate::domain::adapters::prompt::Prompter;
use crate::domain::adapters::Store;
use crate::domain::errors::{Errors, UserInputError};
use crate::domain::models::{ConfigKey, ConfigStatus};
use crate::entry::Interactive;

use super::args::{ConfigAdd, ConfigSet};
use super::Arguments;
use colored::Colorize;

pub fn handler<S: Store, P: Prompter>(
    store: &mut S,
    config_key: &ConfigKey,
    arguments: Arguments,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    local_config_warning(config_key);

    match arguments {
        Arguments::Add(args) => add(args, store),
        Arguments::Set(args) => set(args, store, prompt, interactive),
        Arguments::Reset => reset(store),
        Arguments::Show => list(store),
    }?;

    Ok(())
}

fn add<S: Store>(args: ConfigAdd, store: &S) -> Result<(), Errors> {
    let config = args
        .try_into_domain()
        .map_err(|_| UserInputError::Validation {
            name: "config path".into(),
        })
        .map_err(|e| Errors::UserInput(e))?;

    if config.key == ConfigKey::Default {
        return Err(Errors::UserInput(UserInputError::Validation {
            name: "config key".into(),
        }));
    }

    store.persist_config(&config)?;
    println!("🟢 {} (Active)", config.key.to_string().green());

    Ok(())
}

fn set<S: Store, P: Prompter>(
    args: ConfigSet,
    store: &mut S,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    let key = args.try_into_domain(store, prompt, interactive)?;

    store
        .set_active_config(&key)
        .map_err(|e| Errors::PersistError(e))?;

    println!("🟢 {} (Active)", key.to_string().green());

    Ok(())
}

fn reset<S: Store>(store: &mut S) -> Result<(), Errors> {
    let key = ConfigKey::Default;
    store
        .set_active_config(&key)
        .map_err(|e| Errors::PersistError(e))?;

    println!("🟢 Config reset to {}", key.to_string().green());

    Ok(())
}

fn list<S: Store>(store: &S) -> Result<(), Errors> {
    let mut configurations = store
        .get_configurations()
        .map_err(|e| Errors::PersistError(e))?;

    configurations.sort_by_key(|c| c.status.clone());

    for config in configurations {
        let key = config.key.to_string();
        let path = config.path.display();

        match config.status {
            ConfigStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
            ConfigStatus::Disabled => println!("🔴 {key} ➜ '{path}'"),
        }
    }

    Ok(())
}

fn local_config_warning(config_key: &ConfigKey) {
    let warn_message = match config_key {
        ConfigKey::Once => Some("'once off' --config"),
        ConfigKey::Local => Some("'local' repository"),
        ConfigKey::User(_) | ConfigKey::Default => None,
    };

    if let Some(msg) = warn_message {
        println!(
            "{}: (Active) configurations are currently overridden due to a {} configuration being used.\n",
            "⚠️ Warning".yellow(),
            msg
        );
    }
}
