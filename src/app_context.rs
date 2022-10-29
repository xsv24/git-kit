use anyhow::{anyhow, Context as anyhow_context};
use directories::ProjectDirs;
use rusqlite::Connection;

use crate::domain::commands::GitCommands;

pub struct AppContext<C: GitCommands> {
    pub project_dir: ProjectDirs,
    pub connection: Connection,
    pub commands: C,
}

impl<C: GitCommands> AppContext<C> {
    pub fn new(commands: C) -> anyhow::Result<AppContext<C>> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        let connection = Connection::open(project_dir.config_dir().join("db"))?;

        Ok(AppContext {
            project_dir,
            connection,
            commands,
        })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}
