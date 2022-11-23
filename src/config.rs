use std::{
    any,
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::utils::{file::required_path, get_file_contents};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
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

impl AppConfig {
    pub fn new(user_config_path: Option<PathBuf>, git_root_path: PathBuf) -> anyhow::Result<Self> {
        let config_path =
            Self::get_config_path(user_config_path, git_root_path, Self::config_dir()?)?;

        let config_contents = get_file_contents(&config_path)?;
        let config = serde_yaml::from_str::<AppConfig>(&config_contents)
            .context("Failed to load 'config.yml' from please ensure yaml is valid.")?;

        Ok(config)
    }

    fn config_dir() -> anyhow::Result<PathBuf> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(project_dir.config_dir().to_owned())
    }

    pub fn config_path_default() -> anyhow::Result<PathBuf> {
        Ok(Self::config_dir()?.join(".git-kit.yml"))
    }

    pub fn db_connection() -> anyhow::Result<Connection> {
        let db_file = Self::config_dir()?.join("db");

        let connection = Connection::open(db_file).context("Failed to open sqlite connection")?;

        Ok(connection)
    }

    pub fn validate_template(&self, name: &str) -> clap::error::Result<()> {
        log::info!("validating template {}", name);

        if self.commit.templates.contains_key(name) {
            log::info!("template {} 👌", name);
            Ok(())
        } else {
            // TODO: want a nice error message that shows the templates output
            Err(clap::Error::raw(
                clap::error::ErrorKind::InvalidSubcommand,
                format!("Found invalid subcommand '{}' given", name),
            ))?
        }
    }

    pub fn get_template_config(&self, name: &str) -> clap::error::Result<&TemplateConfig> {
        log::info!("fetching template {}", name);
        let template = self.commit.templates.get(name).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                format!("Found missing subcommand '{}'", name),
            )
        })?;

        Ok(template)
    }

    fn get_config_path(
        user_config: Option<PathBuf>,
        repo_config: PathBuf,
        default_path: PathBuf,
    ) -> anyhow::Result<PathBuf> {
        let filename = ".git-kit.yml";
        let repo_config = repo_config.join(filename);
        let default_path = default_path.join(filename);

        match (user_config, repo_config) {
            (Some(user), _) => {
                log::info!("⏳ Loading user config...");

                let path = required_path(&user).map_err(|_| {
                    anyhow::anyhow!(format!(
                        "Invalid config file path does not exist at '{}'",
                        &user.display()
                    ))
                })?;

                Ok(path.to_owned())
            }
            (None, repo) if repo.exists() => {
                log::info!("⏳ Loading local repo config...");
                Ok(repo)
            }
            (_, _) => {
                log::info!("⏳ Loading global config...");
                Ok(default_path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use uuid::Uuid;

    use crate::{adapters::Git, domain::adapters};

    use super::*;

    fn fake_project_dir() -> PathBuf {
        let dir = ProjectDirs::from("test", "xsv24", &format!("{}", Uuid::new_v4()))
            .expect("Failed to retrieve 'git-kit' config");

        dir.config_dir().to_owned()
    }

    #[test]
    fn no_user_path_or_valid_repo_dir_defaults() -> anyhow::Result<()> {
        let default_path = fake_project_dir();

        let repo_non_existing = Path::new(&Faker.fake::<String>()).to_owned();

        let config_dir = AppConfig::get_config_path(None, repo_non_existing, default_path.clone())?;

        assert_eq!(config_dir, default_path.join(".git-kit.yml"));
        Ok(())
    }

    #[test]
    fn repo_dir_with_config_file_used_over_default() -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;
        let repo_root_with_config = git.root_directory()?;

        let config_dir =
            AppConfig::get_config_path(None, repo_root_with_config.clone(), fake_project_dir())?;

        assert_eq!(config_dir, repo_root_with_config.join(".git-kit.yml"));
        Ok(())
    }

    #[test]
    fn user_config_file_used_over_repo_and_default() -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;

        let user_config = Path::new(".").to_owned();

        let config_dir = AppConfig::get_config_path(
            Some(user_config.clone()),
            git.root_directory()?,
            fake_project_dir(),
        )?;

        assert_eq!(config_dir, user_config);

        Ok(())
    }

    #[test]
    fn invalid_path_for_user_config_file_errors() -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;

        let user_config = Path::new(&Faker.fake::<String>()).to_owned();

        let error = AppConfig::get_config_path(
            Some(user_config.clone()),
            git.root_directory()?,
            fake_project_dir(),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "Invalid config file path does not exist at '{}'",
                user_config.display()
            )
        );

        Ok(())
    }
}
