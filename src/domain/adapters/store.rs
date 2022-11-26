use crate::domain::models::{Branch, Config};

pub trait Store {
    fn persist_branch(&self, branch: &Branch) -> anyhow::Result<()>;

    fn get_branch(&self, branch: &str, repo: &str) -> anyhow::Result<Branch>;

    fn persist_config(&self, config: &Config) -> anyhow::Result<()>;

    fn set_active_config(&mut self, key: String) -> anyhow::Result<Config>;

    fn get_config(&self, key: Option<String>) -> anyhow::Result<Config>;

    fn close(self) -> anyhow::Result<()>;
}
