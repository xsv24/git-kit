use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::utils::{expected_path, get_file_contents};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub commit: CommitConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitConfig {
    pub templates: HashMap<String, TemplateConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub description: String,
    pub content: String,
}

impl Config {
    pub fn new(
        user_config_path: Option<String>,
        git_root_path: PathBuf,
        default_path: &Path,
    ) -> anyhow::Result<Self> {
        let config_path = Self::get_config_path(user_config_path, git_root_path, default_path)?;

        let config_contents = get_file_contents(&config_path)?;
        let config = serde_yaml::from_str::<Config>(&config_contents)
            .context("Failed to load 'config.yml' from please ensure yaml is valid.")?;

        Ok(config)
    }

    pub fn validate_template(&self, name: &str) -> clap::Result<()> {
        if self.commit.templates.contains_key(name) {
            Ok(())
        } else {
            // TODO: want a nice error message that shows the templates output
            Err(clap::Error::raw(
                clap::ErrorKind::InvalidSubcommand,
                format!("Found invalid subcommand '{}' given", name),
            ))?
        }
    }

    pub fn get_template_config(&self, name: &str) -> clap::Result<&TemplateConfig> {
        let template = self.commit.templates.get(name).ok_or_else(|| {
            clap::Error::raw(
                clap::ErrorKind::MissingSubcommand,
                format!("Found missing subcommand '{}'", name),
            )
        })?;

        Ok(template)
    }

    fn get_config_path(
        user_config: Option<String>,
        repo_config: PathBuf,
        default_path: &Path,
    ) -> anyhow::Result<PathBuf> {
        let filename = ".git-kit.yml";
        let repo_config = repo_config.join(filename);
        let default_path = default_path.join(filename);

        match (user_config, repo_config) {
            (Some(user), _) => {
                println!("⏳ Loading user config...");

                expected_path(&user).map_err(|_| {
                    anyhow::anyhow!(format!(
                        "Invalid config file path does not exist at '{}'",
                        &user
                    ))
                })
            }
            (None, repo) if repo.exists() => {
                println!("⏳ Loading local repo config...");
                Ok(repo)
            }
            (_, _) => {
                println!("⏳ Loading global config...");
                Ok(default_path)
            }
        }
    }
}
