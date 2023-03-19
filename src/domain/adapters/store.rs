use crate::domain::{
    errors::{Errors, PersistError},
    models::{Branch, Config, ConfigKey},
};

pub trait Store {
    fn persist_branch(&self, branch: &Branch) -> anyhow::Result<()>;

    fn get_branch(&self, branch: &str, repo: &str) -> anyhow::Result<Branch>;

    fn persist_config(&self, config: &Config) -> Result<(), Errors>;

    fn set_active_config(&mut self, key: &ConfigKey) -> Result<Config, PersistError>;

    fn get_configurations(&self) -> anyhow::Result<Vec<Config>>;

    fn get_configuration(&self, key: Option<String>) -> anyhow::Result<Config>;

    fn close(self) -> anyhow::Result<()>;
}
