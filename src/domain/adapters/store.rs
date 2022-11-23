use crate::domain::models::Branch;

pub trait Store {
    fn persist_branch(&self, branch: &Branch) -> anyhow::Result<()>;

    fn get_branch(&self, branch: &str, repo: &str) -> anyhow::Result<Branch>;

    fn close(self) -> anyhow::Result<()>;
}
