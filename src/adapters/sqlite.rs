use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Connection, Row, Transaction};

use crate::{
    domain::{
        self,
        errors::{Errors, PersistError},
        models::{Branch, Config, ConfigKey, ConfigStatus},
    },
    utils::TryConvert,
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
                "REPLACE INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                    &branch.link,
                    &branch.scope
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
            "SELECT name, ticket, data, created, link, scope FROM branch where name = ?",
            [name],
            |row| Branch::try_from(row),
        )?;

        Ok(branch)
    }

    fn persist_config(&self, config: &Config) -> Result<(), Errors> {
        // TODO: Move this to the domain
        if config.key == ConfigKey::Default {
            return Err(Errors::ValidationError {
                message: "Cannot override 'default' config!".into(),
            });
        }

        let key: String = config.key.clone().into();
        let path = (&config.path)
            .try_convert()
            .map_err(|_| Errors::ValidationError {
                message: "Failed to convert path to string".into(),
            })?;

        log::info!("insert or update user config '{}' path '{}'", &key, &path);

        self.connection
            .execute(
                "REPLACE INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                (key, path, String::from(ConfigStatus::Active)),
            )
            .map_err(|e| PersistError::into("config", "Failed to update config.", e))
            .map_err(|e| Errors::PersistError(e))?;

        Ok(())
    }

    fn set_active_config(&mut self, key: &ConfigKey) -> Result<Config, PersistError> {
        let transaction = self.transaction().map_err(|e| PersistError::Other(e))?;

        let key: String = key.to_owned().into();

        let (active, disabled) = (
            String::from(ConfigStatus::Active),
            String::from(ConfigStatus::Disabled),
        );

        // Check the record actually exists before changing statuses.
        transaction
            .query_row("SELECT * FROM config where key = ?1;", [&key], |_| Ok(()))
            .map_err(|e| {
                PersistError::into(
                    "config",
                    format!("Configuration '{key}' does not exist."),
                    e,
                )
            })?;

        // Update any 'ACTIVE' config to 'DISABLED'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE status = ?2;",
                (&disabled, &active),
            )
            .map_err(|e| {
                PersistError::into(
                    "config",
                    format!("Failed to set any '{disabled}' config to '{active}'."),
                    e,
                )
            })?;

        // Update the desired config to 'ACTIVE'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE key = ?2;",
                (&active, &key),
            )
            .map_err(|e| {
                PersistError::into(
                    "config",
                    format!("Failed to update config status to '{active}'."),
                    e,
                )
            })?;

        transaction.commit().map_err(|e| {
            PersistError::into("config", "Failed to commit transaction to update config", e)
        })?;

        self.get_configuration(Some(key))
            .map_err(|e| PersistError::Validation {
                name: "config".into(),
                source: e.into(),
            })
    }

    fn get_configuration(&self, key: Option<String>) -> Result<Config, PersistError> {
        let config = match key {
            Some(key) => self
                .connection
                .query_row("SELECT * FROM config WHERE key = ?1", [key], |row| {
                    Config::try_from(row)
                })
                .map_err(|e| PersistError::into("config", "Failed to retrieve config '{key}'.", e)),
            None => self
                .connection
                .query_row(
                    "SELECT * FROM config WHERE status = ?1",
                    [String::from(ConfigStatus::Active)],
                    |row| Config::try_from(row),
                )
                .map_err(|e| {
                    PersistError::into("config", "Failed to retrieve 'active' config.", e)
                }),
        };

        Ok(config?)
    }

    fn get_configurations(&self) -> Result<Vec<Config>, PersistError> {
        let mut statement = self
            .connection
            .prepare("SELECT * FROM config")
            .map_err(|e| PersistError::into("config", "Failed to retrieve configs", e))?;

        let configs: Vec<_> = statement
            .query_map([], |row| {
                let config = Config::try_from(row)?;
                Ok(config)
            })
            .map_err(|e| PersistError::Corrupted {
                name: "config".into(),
                source: Some(e.into()),
            })?
            .into_iter()
            .collect::<Result<_, _>>()
            .map_err(|e| PersistError::Corrupted {
                name: "config".into(),
                source: Some(e.into()),
            })?;

        Ok(configs)
    }

    fn close(self) -> anyhow::Result<()> {
        log::info!("closing sqlite connection");

        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}

impl<'a> TryFrom<&Row<'a>> for Config {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let status = value.get::<_, String>(2)?.try_into().map_err(|e| {
            log::error!(
                "Corrupted data failed to convert to valid config status, {}",
                e
            );
            rusqlite::Error::InvalidColumnType(
                2,
                "Failed to convert to valid config status".into(),
                Type::Text,
            )
        })?;

        let config = Config::new(
            ConfigKey::from(value.get::<_, String>(0)?),
            value.get::<_, String>(1)?,
            status,
        )
        .map_err(|e| {
            log::error!("Corrupted data failed to convert 'Config', {}", e);

            rusqlite::Error::ExecuteReturnedResults
        })?;

        Ok(config)
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
            link: value.get(4)?,
            scope: value.get(5)?,
        };

        Ok(branch)
    }
}

impl PersistError {
    fn into<S>(name: &str, message: S, error: rusqlite::Error) -> PersistError
    where
        S: Into<String>,
    {
        log::error!("{}\n{}", message.into(), &error);

        match error {
            rusqlite::Error::InvalidPath(_) => PersistError::Configuration(error.into()),
            rusqlite::Error::ExecuteReturnedResults
            | rusqlite::Error::SqliteSingleThreadedMode
            | rusqlite::Error::StatementChangedRows(_)
            | rusqlite::Error::MultipleStatement
            | rusqlite::Error::IntegralValueOutOfRange(_, _)
            | rusqlite::Error::InvalidQuery
            | rusqlite::Error::NulError(_)
            | rusqlite::Error::Utf8Error(_)
            | rusqlite::Error::ToSqlConversionFailure(_) => PersistError::Validation {
                name: name.into(),
                source: error.into(),
            },
            rusqlite::Error::QueryReturnedNoRows => PersistError::NotFound,
            rusqlite::Error::FromSqlConversionFailure(_, _, _)
            | rusqlite::Error::InvalidParameterCount(_, _)
            | rusqlite::Error::InvalidColumnIndex(_)
            | rusqlite::Error::InvalidParameterName(_)
            | rusqlite::Error::InvalidColumnName(_)
            | rusqlite::Error::InvalidColumnType(_, _, _)
            | rusqlite::Error::SqlInputError { .. } => PersistError::Corrupted {
                name: name.into(),
                source: Some(error.into()),
            },
            rusqlite::Error::SqliteFailure(_, _) | _ => PersistError::Other(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use crate::entry::Interactive;
    use crate::{adapters::Git, app_context::AppContext, domain::adapters::Store};

    use crate::migrations::{db_migrations, MigrationContext};

    use super::*;
    use fake::{Fake, Faker};

    #[test]
    fn persist_branch_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite::new(connection)?;

        // Act
        store.persist_branch(&branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let actual_branch = select_branch_row(&store.connection)?;

        assert_eq!(branch, actual_branch);

        Ok(())
    }

    #[test]
    fn persist_branch_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite { connection };

        store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &branch.name,
                &branch.ticket,
                &branch.data,
                &branch.created.to_rfc3339(),
                &branch.link,
                &branch.scope
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

        let actual_branch = select_branch_row(&store.connection)?;

        assert_eq!(updated_branch, actual_branch);

        Ok(())
    }

    #[test]
    fn get_branch_retrieves_correct_branch() -> anyhow::Result<()> {
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
                "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                    &branch.link,
                    &branch.scope
                ),
            ).unwrap();
        }

        let context = AppContext {
            store,
            git: Git,
            config: fake_config(),
            interactive: Interactive::Enable,
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
        assert_eq!(random_branch.to_owned(), branch);

        Ok(())
    }

    #[test]
    fn get_branch_trims_branch_name_before_retrieving() -> anyhow::Result<()> {
        // Arrange
        let context = AppContext {
            store: Sqlite::new(setup_db()?)?,
            git: Git,
            config: fake_config(),
            interactive: Interactive::Enable,
        };

        // Insert random collection of branches.
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let expected = fake_branch(Some(name.clone()), Some(repo.clone()))?;

        context.store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &expected.name,
                &expected.ticket,
                &expected.data,
                &expected.created.to_rfc3339(),
                &expected.link,
                &expected.scope
            ),
        )?;

        // Act
        let actual = context.store.get_branch(&format!(" {}\n", name), &repo)?;

        context.close()?;
        // Assert
        assert_eq!(actual.name, expected.name);

        Ok(())
    }

    #[test]
    fn persist_config_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let config = fake_config();
        let connection = setup_db()?;
        let store = Sqlite::new(connection)?;

        // Act
        store.persist_config(&config)?;

        // Assert
        assert_eq!(config_count(&store.connection)?, 1);

        let expected = select_config_row(&store.connection, config.key.clone().into())?;

        assert_eq!(expected, config);

        Ok(())
    }

    #[test]
    fn persist_config_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let config = fake_config();
        let connection = setup_db()?;
        insert_config(&connection, &config)?;

        let store = Sqlite::new(connection)?;

        // Act
        let config = Config {
            key: config.key,
            ..fake_config()
        };
        store.persist_config(&config)?;

        // Assert
        assert_eq!(config_count(&store.connection)?, 1);

        let actual = select_config_row(&store.connection, config.key.clone().into())?;
        assert_eq!(actual, actual);

        Ok(())
    }

    #[test]
    fn get_list_of_registered_configs() -> anyhow::Result<()> {
        // Arrange
        let connection = setup_db()?;
        let expected = vec![fake_config(), fake_config(), fake_config()];

        for config in &expected {
            insert_config(&connection, config)?;
        }

        let store = Sqlite::new(connection)?;

        // Act
        let configs = store.get_configurations()?;

        assert_eq!(expected, configs);

        Ok(())
    }

    #[test]
    fn get_config_by_key_success() -> anyhow::Result<()> {
        // Arrange
        let expected = fake_config();
        let connection = setup_db()?;
        insert_config(&connection, &expected)?;

        let store = Sqlite::new(connection)?;

        // Act
        let config = store
            .get_configuration(Some(expected.key.clone().into()))
            .unwrap();

        // Assert
        assert_eq!(1, config_count(&store.connection)?);
        assert_eq!(expected, config);
        Ok(())
    }

    #[test]
    fn get_config_by_active() -> anyhow::Result<()> {
        // Arrange
        let expected = Config {
            status: ConfigStatus::Active,
            ..fake_config()
        };
        let connection = setup_db()?;
        insert_config(&connection, &expected)?;
        let store = Sqlite::new(connection)?;

        // Act
        let config = store.get_configuration(None).unwrap();

        // Assert
        assert_eq!(1, config_count(&store.connection)?);
        assert_eq!(expected, config);
        Ok(())
    }

    #[test]
    fn set_active_config_success() -> anyhow::Result<()> {
        let mut original = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };
        let connection = setup_db()?;
        insert_config(&connection, &original)?;

        let mut store = Sqlite::new(connection)?;
        let actual = store.set_active_config(&original.key.clone())?;

        original.status = ConfigStatus::Active;
        assert_eq!(original, actual);

        Ok(())
    }

    #[test]
    fn set_active_checks_row_exists_before_clearing_status() -> anyhow::Result<()> {
        let connection = setup_db()?;
        let mut store = Sqlite::new(connection)?;

        let active_config = Config {
            status: ConfigStatus::Active,
            ..fake_config()
        };

        insert_config(&store.connection, &active_config)?;

        let result = store.set_active_config(&ConfigKey::User(Faker.fake()));
        assert!(result.is_err());

        let default = store.get_configuration(Some(active_config.key.clone().into()))?;
        assert_eq!(active_config.key, default.key);

        Ok(())
    }

    #[test]
    fn set_active_config_sets_any_active_configs_to_disabled() -> anyhow::Result<()> {
        let connection = setup_db()?;

        for _ in 0..(2..10).fake() {
            let config = Config {
                status: ConfigStatus::Active,
                ..fake_config()
            };

            insert_config(&connection, &config)?;
        }

        let original = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };

        insert_config(&connection, &original)?;

        let mut store = Sqlite::new(connection)?;
        // Act
        store.set_active_config(&original.key)?;

        let configs = select_all_config(&store.connection)?;

        let active: Vec<Config> = configs
            .into_iter()
            .filter(|c| c.status == ConfigStatus::Active)
            .collect();

        assert_eq!(active.len(), 1);
        let only_active = active.first().unwrap();
        let expected = Config {
            status: ConfigStatus::Active,
            ..original
        };
        assert_eq!(&expected, only_active);

        Ok(())
    }

    fn fake_config() -> Config {
        let path = Path::new(".").to_owned();
        let absolute_path = std::fs::canonicalize(path).expect("Valid conversion to absolute path");

        Config {
            key: ConfigKey::User(Faker.fake()),
            path: absolute_path,
            status: ConfigStatus::Active,
        }
    }

    fn insert_config(connection: &Connection, config: &Config) -> anyhow::Result<()> {
        let key: String = config.key.clone().into();

        connection.execute(
            "INSERT INTO config (key, path, status) VALUES (?1, ?2, ?3)",
            (
                &key,
                (&config.path).try_convert()?,
                String::from(config.status.clone()),
            ),
        )?;

        Ok(())
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());

        Ok(Branch::new(
            &name,
            &repo,
            Faker.fake(),
            Faker.fake(),
            Faker.fake(),
        ))
    }

    fn select_branch_row(conn: &Connection) -> anyhow::Result<Branch> {
        let (name, ticket, data, created, link, scope) =
            conn.query_row("SELECT * FROM branch", [], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;
        let created = DateTime::parse_from_rfc3339(&created)?.with_timezone(&Utc);

        Ok(Branch {
            name,
            ticket,
            data,
            created,
            link,
            scope,
        })
    }

    fn select_config_row(conn: &Connection, key: String) -> anyhow::Result<Config> {
        let path = conn.query_row("SELECT * FROM config where key = ?1;", [key], |row| {
            Config::try_from(row)
        })?;

        Ok(path)
    }

    fn select_all_config(conn: &Connection) -> anyhow::Result<Vec<Config>> {
        let mut statement = conn.prepare("SELECT * FROM config")?;
        let configs = statement
            .query_map([], |row| Config::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(configs)
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
            MigrationContext {
                default_configs: None,
                version: None,
            },
        )?;
        Ok(conn)
    }
}
