use anyhow::Context;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

pub fn db_migrations(connection: &mut Connection, version: Option<usize>) -> anyhow::Result<Migrations> {
    // let config_path_default = config_path_default.to_str()
    //     .expect("Failed to retrieve default config");

    // let config_path_default = format!("INSERT INTO config (path) VALUES '{}'", config_path_default);
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE IF NOT EXISTS branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            );"
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS config (
                path TEXT NOT NULL
            );"
        ),
    ]);

    if let Some(version) = version {
        migrations
            .to_version(connection, version)
            .with_context(|| format!("Failed to apply migration version '{}'", version))?;
    } else {
        migrations
            .to_latest(connection)
            .context("Failed to apply latest migration")?;
    };

    let version = migrations
        .current_version(&connection)
        .context("Failed to get current migration version.")?;

    println!("git-kit migration version '{}'.", version);

    Ok(migrations)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db_migrations;

    #[test]
    fn verify_migrations() {
        let mut connection = Connection::open_in_memory().unwrap();
        let migrations = db_migrations(&mut connection, None).unwrap();

        // Validate migrations where applied.
        assert!(migrations.validate().is_ok());

        let mut stmt = connection.prepare("SELECT name FROM sqlite_schema WHERE type='table'")
            .unwrap();
        
        let iter = stmt.query_map([], |row| {
            let table: String = row.get(0)?;
            Ok(table)
        }).unwrap();

        let tables = iter.collect::<Vec<Result<String, _>>>();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&Ok("branch".to_string())));
        assert!(tables.contains(&Ok("config".to_string())));
    }
}
