use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum CheckoutStatus {
    New,
    Existing,
}

/// Used to abstract cli git commands for testing.
pub trait Git {
    /// Get the root directory of the current git repo.
    fn root_directory(&self) -> anyhow::Result<PathBuf>;

    /// Get the current git repository name.
    fn get_repo_name(&self) -> anyhow::Result<String>;

    /// Get the current checked out branch name.
    fn get_branch_name(&self) -> anyhow::Result<String>;

    /// Checkout an existing branch of create a new branch if not.
    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()>;

    /// Commit changes and open editor with the template.
    fn commit(&self, msg: &str) -> anyhow::Result<()>;

    /// Get the commit file path for the current repository.
    fn template_file_path(&self) -> anyhow::Result<PathBuf>;

    /// Commit changes and open and editor with template file.
    fn commit_with_template(&self, template: &Path) -> anyhow::Result<()>;
}
