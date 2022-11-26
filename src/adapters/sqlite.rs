use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Connection, Row, Transaction};

use crate::{
    domain::{
        self,
        models::{Branch, Config, ConfigStatus},
    },
    utils::expected_path,
};

pub struct Sqlite {
    connection: Connection,
}

impl Sqlite {
    pub fn new(connection: Connection) -> anyhow::Result<Sqlite> {
        Ok(Sqlite { connection })
    }

    pub fn transaction(&mut self) -> anyhow::Result<Transaction> {
        let transaction = self
            .connection
            .transaction()
            .context("Failed to open transaction for sqlite db.")?;

        Ok(transaction)
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}

impl domain::adapters::Store for Sqlite {
    fn persist_branch(&self, branch: &Branch) -> anyhow::Result<()> {
        log::info!(
            "insert or update for '{}' branch with ticket '{}'",
            branch.name,
            branch.ticket
        );

        self.connection
            .execute(
                "REPLACE INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                ),
            )
            .with_context(|| format!("Failed to update branch '{}'", &branch.name))?;

        Ok(())
    }

    fn get_branch(&self, branch: &str, repo: &str) -> anyhow::Result<Branch> {
        let name = format!("{}-{}", repo.trim(), branch.trim());

        log::info!(
            "retrieve branch with ticket for branch '{}' and repo '{}'",
            name,
            repo
        );

        let branch = self.connection.query_row(
            "SELECT name, ticket, data, created FROM branch where name = ?",
            [name],
            |row| Branch::try_from(row),
        )?;

        Ok(branch)
    }

    fn persist_config(&self, config: &Config) -> anyhow::Result<()> {
        if config.key.to_string().eq("default") {
            anyhow::bail!("Cannot override default!");
        }

        log::info!(
            "insert or update user config '{}' path '{}'",
            &config.key,
            &config.path.display()
        );

        let path = &config.path.to_str().context("Invalid valid path given")?;

        self.connection
            .execute(
                "REPLACE INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                (config.key.clone(), path, String::from(ConfigStatus::ACTIVE)),
            )
            .context("Failed to update config.")?;

        Ok(())
    }

    fn set_active_config(&mut self, key: String) -> anyhow::Result<Config> {
        let transaction = self.transaction()?;

        let (active, inactive) = (
            String::from(ConfigStatus::ACTIVE),
            String::from(ConfigStatus::INACTIVE),
        );

        // Update any 'ACTIVE' config to 'INACTIVE'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE status = ?2;",
                (&inactive, &active),
            )
            .with_context(|| format!("Failed to set any '{}' config to '{}'.", inactive, active))?;

        // Update the desired config to 'ACTIVE'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE key = ?2;",
                (&active, &key),
            )
            .with_context(|| format!("Failed to update config status to '{}'.", active))?;

        transaction
            .commit()
            .context("Failed to commit transaction to update config")?;

        self.get_config(Some(key))
    }

    fn close(self) -> anyhow::Result<()> {
        log::info!("closing sqlite connection");

        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }

    fn get_config(&self, key: Option<String>) -> anyhow::Result<Config> {
        let config = match key {
            Some(key) => {
                self.connection
                    .query_row("SELECT * FROM config WHERE key = ?1", [key], |row| {
                        Config::try_from(row)
                    })
            }
            None => self.connection.query_row(
                "SELECT * FROM config WHERE status = ?1",
                [String::from(ConfigStatus::ACTIVE)],
                |row| Config::try_from(row),
            ),
        }?;

        dbg!(&config);
        Ok(config)
    }
}

impl<'a> TryFrom<&Row<'a>> for Config {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let path = expected_path(&value.get::<_, String>(1)?).map_err(|e| {
            log::error!("Corrupted data failed to convert to valid path, {}", e);
            rusqlite::Error::InvalidColumnType(
                1,
                "Failed to convert to valid path".into(),
                Type::Text,
            )
        })?;

        let status = value.get::<_, String>(2)?.try_into().map_err(|e| {
            log::error!(
                "Corrupted data failed to convert to valid config status, {}",
                e
            );
            rusqlite::Error::InvalidColumnType(
                2,
                "Failed to convert to 'ConfigStatus'".into(),
                Type::Text,
            )
        })?;

        Ok(Config {
            key: value.get(0)?,
            path,
            status,
        })
    }
}

impl<'a> TryFrom<&Row<'a>> for Branch {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let date = value.get::<usize, String>(3)?;
        let created = DateTime::parse_from_rfc3339(&date)
            .map_err(|e| {
                log::error!("Corrupted data failed to convert to datetime, {}", e);
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Failed to convert string to DateTime".into(),
                    Type::Text,
                )
            })?
            .with_timezone(&Utc);

        let branch = Branch {
            name: value.get(0)?,
            ticket: value.get(1)?,
            data: value.get(2)?,
            created,
        };

        Ok(branch)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

    use crate::{
        adapters::Git,
        app_context::AppContext,
        config::{AppConfig, CommitConfig},
        domain::adapters::Store,
    };

    use super::*;
    use fake::{Fake, Faker};
    use migrations::db_migrations;

    #[test]
    fn update_branch_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite::new(connection)?;

        // Act
        store.persist_branch(&branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let (name, ticket, data, created) = select_branch_row(&store.connection)?;

        assert_eq!(branch.name, name);
        assert_eq!(branch.ticket, ticket);
        assert_eq!(branch.data, data);
        assert_eq!(branch.created.to_rfc3339(), created);

        Ok(())
    }

    #[test]
    fn update_config_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let config = fake_config();
        let connection = setup_db()?;
        let store = Sqlite::new(connection)?;

        // Act
        store.persist_config(&config)?;

        // Assert
        assert_eq!(config_count(&store.connection)?, 1);

        let (key, path) = select_config_row(&store.connection, config.key.clone())?;
        let expected = Config {
            key,
            path: Path::new(&path).to_owned(),
            status: ConfigStatus::ACTIVE,
        };

        assert_eq!(expected, config);

        Ok(())
    }

    #[test]
    fn insert_or_update_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite { connection };

        store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &branch.name,
                &branch.ticket,
                &branch.data,
                &branch.created.to_rfc3339(),
            ),
        )?;

        let updated_branch = Branch {
            name: branch.name,
            ..fake_branch(None, None)?
        };

        // Act
        store.persist_branch(&updated_branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let (name, ticket, data, created) = select_branch_row(&store.connection)?;

        assert_eq!(updated_branch.name, name);
        assert_eq!(updated_branch.ticket, ticket);
        assert_eq!(updated_branch.data, data);
        assert_eq!(updated_branch.created.to_rfc3339(), created);

        Ok(())
    }

    #[test]
    fn get_retrieves_correct_branch() -> anyhow::Result<()> {
        // Arrange
        let connection = setup_db()?;
        let store = Sqlite { connection };

        let mut branches: HashMap<String, Branch> = HashMap::new();
        let repo = Faker.fake::<String>();

        // Insert random collection of branches.
        for _ in 0..(2..10).fake() {
            let name = Faker.fake::<String>();
            let branch = fake_branch(Some(name.clone()), Some(repo.clone()))?;
            branches.insert(name, branch.clone());

            store.connection.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                ),
            )?;
        }

        let context = AppContext {
            store,
            git: Git,
            config: fake_app_config(),
        };

        let keys = branches.keys().cloned().collect::<Vec<String>>();

        let random_key = keys
            .get((0..keys.len() - 1).fake::<usize>())
            .with_context(|| "Expected to find a matching branch")?;

        let random_branch = branches
            .get(random_key)
            .with_context(|| "Expected to find a matching branch")?;

        // Act
        let branch = context.store.get_branch(&random_key, &repo)?;

        context.close()?;
        // Assert
        assert_eq!(random_branch.name, branch.name);
        assert_eq!(random_branch.ticket, branch.ticket);
        assert_eq!(random_branch.data, branch.data);
        assert_eq!(
            random_branch.created.to_rfc3339(),
            branch.created.to_rfc3339()
        );

        Ok(())
    }

    #[test]
    fn get_trims_name_before_retrieving() -> anyhow::Result<()> {
        // Arrange
        let context = AppContext {
            store: Sqlite::new(setup_db()?)?,
            git: Git,
            config: fake_app_config(),
        };

        // Insert random collection of branches.
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let expected = fake_branch(Some(name.clone()), Some(repo.clone()))?;

        context.store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &expected.name,
                &expected.ticket,
                &expected.data,
                &expected.created.to_rfc3339(),
            ),
        )?;

        // Act
        let actual = context.store.get_branch(&format!(" {}\n", name), &repo)?;

        context.close()?;
        // Assert
        assert_eq!(actual.name, expected.name);

        Ok(())
    }

    fn fake_app_config() -> AppConfig {
        AppConfig {
            commit: CommitConfig {
                templates: HashMap::new(),
            },
        }
    }

    fn fake_config() -> Config {
        Config {
            key: Faker.fake(),
            path: PathBuf::new(),
            status: ConfigStatus::ACTIVE,
        }
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());
        let ticket: Option<String> = Faker.fake();

        Ok(Branch::new(&name, &repo, ticket)?)
    }

    fn select_branch_row(
        conn: &Connection,
    ) -> anyhow::Result<(String, String, Option<Vec<u8>>, String)> {
        let (name, ticket, data, created) = conn.query_row("SELECT * FROM branch", [], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<Vec<u8>>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        Ok((name, ticket, data, created))
    }

    fn select_config_row(conn: &Connection, key: String) -> anyhow::Result<(String, String)> {
        let path = conn.query_row("SELECT * FROM config where key = ?1;", [key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        Ok(path)
    }

    fn branch_count(conn: &Connection) -> anyhow::Result<i32> {
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM branch", [], |row| row.get(0))?;

        Ok(count)
    }

    fn config_count(conn: &Connection) -> anyhow::Result<i32> {
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM config", [], |row| row.get(0))?;

        Ok(count)
    }

    fn setup_db() -> anyhow::Result<Connection> {
        let mut conn = Connection::open_in_memory()?;
        db_migrations(
            &mut conn,
            migrations::MigrationContext {
                config_path: Path::new(".").to_owned(),
                enable_side_effects: false,
                version: None,
            },
        )?;
        Ok(conn)
    }
}
