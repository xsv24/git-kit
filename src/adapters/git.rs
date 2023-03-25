use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    domain::{
        adapters::{self, CheckoutStatus, CommitMsgStatus},
        errors::GitError,
    },
    utils::TryConvert,
};

pub struct Git;

impl Git {
    fn command(args: &[&str]) -> Command {
        // TODO: Mock the system Command struct so we can have better tests.
        let mut comm = Command::new("git");

        comm.args(args);

        comm
    }
}

impl adapters::Git for Git {
    fn repository_name(&self) -> Result<String, GitError> {
        let repo_dir = self.root_directory()?.try_convert().map_err(|e| {
            log::error!("Failed to get repository name: {}", e);
            GitError::RootDirectory
        })?;

        let repo = repo_dir.split('/').last().ok_or_else(|| {
            log::error!("Failed to get repository name");
            GitError::RootDirectory
        })?;

        log::info!("git repository name '{}'", repo);

        Ok(repo.trim().into())
    }

    fn branch_name(&self) -> Result<String, GitError> {
        let branch: String = Git::command(&["branch", "--show-current"])
            .try_convert()
            .map_err(|e| {
                log::error!("Failed to get current branch name: {}", e);
                GitError::BranchName
            })?;

        log::info!("current git branch name '{}'", branch);

        Ok(branch)
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> Result<(), GitError> {
        log::info!("checkout '{:?}' branch", status);

        let mut command = match status {
            CheckoutStatus::New => Git::command(&["checkout", "-b", name]),
            CheckoutStatus::Existing => Git::command(&["checkout", name]),
        };

        command.status().map_err(|e| {
            log::error!("Failed to checkout branch: {}", e);
            GitError::Checkout { name: name.into() }
        })?;

        Ok(())
    }

    fn root_directory(&self) -> Result<PathBuf, GitError> {
        let dir = Git::command(&["rev-parse", "--show-toplevel"])
            .try_convert()
            .map_err(|e| {
                log::error!("Failed to get root directory : {}", e);
                GitError::RootDirectory
            })?;

        log::info!("git root directory {}", dir);

        Ok(Path::new(dir.trim()).to_owned())
    }

    fn template_file_path(&self) -> Result<PathBuf, GitError> {
        // Template file and stored in the .git directory to avoid users having to adding to their .gitignore
        // In future maybe we could make our own .git-kit dir to house config / templates along with this.
        let path = self
            .root_directory()?
            .join(".git")
            .join("GIT_KIT_COMMIT_TEMPLATE");

        Ok(path)
    }

    fn commit_with_template(
        &self,
        template: &Path,
        completed: CommitMsgStatus,
    ) -> Result<(), GitError> {
        log::info!("commit template with CommitMsgStatus: '{:?}'", completed);

        let template = template
            .as_os_str()
            .to_str()
            .ok_or_else(|| GitError::Validation {
                message: "Failed to convert path to str.".into(),
            })?;

        let mut args = vec!["commit", "--template", template];

        // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
        // Otherwise git will just abort the commit if theres no difference / change from the template.
        if completed == CommitMsgStatus::Completed {
            log::info!("allowing an empty message on commit");
            args.push("--allow-empty-message");
        }

        Git::command(&args).status().map_err(|e| {
            log::error!("Failed to commit template: {}", e);
            GitError::Commit
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::adapters::Git;

    use super::*;

    #[test]
    fn get_repo_name_returns_this_repo_name() -> anyhow::Result<()> {
        let git = Git;

        // TODO: Find a more testable approach to check stdout maybe?
        assert_eq!(git.repository_name()?, "git-kit");

        Ok(())
    }
}
