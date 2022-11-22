use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

// Build script to copy over default templates & config into the binary directory.
fn main() {
    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "git-kit") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config_dir = dirs.config_dir();

        println!("Updating config file...");

        // Create config dir if not exists.
        fs::create_dir(config_dir).ok();

        copy_or_replace(
            &project_root.join(".git-kit.yml"),
            &config_dir.join(".git-kit.yml"),
        )
        .expect("Failed to copy or update to the latest config file for git-kit");

        db_migrations(config_dir);
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn db_migrations(path: &Path) {
    let mut connection =
        Connection::open(path.join("db")).expect("Failed to open sqlite connection");

    let migrations = Migrations::new(vec![
        M::up(include_str!("./migrations/1_initial/up.sql")),
        M::up(include_str!("./migrations/2_add_config/up.sql"))
            .down(include_str!("./migrations/2_add_config/down.sql")),
    ]);

    migrations
        .to_latest(&mut connection)
        .expect("Failed to apply latest migration");

    let version = migrations
        .current_version(&connection)
        .expect("Failed to get current migration version.");

    println!("git-kit migration version '{}'.", version);

    migrations
        .validate()
        .expect("Failed validating migrations.")
}

fn copy_or_replace(source_path: &PathBuf, target_path: &PathBuf) -> io::Result<()> {
    match fs::read_dir(source_path) {
        Ok(entry_iter) => {
            fs::create_dir_all(target_path)?;
            for dir in entry_iter {
                let entry = dir?;
                copy_or_replace(&entry.path(), &target_path.join(entry.file_name()))?;
            }
        }
        Err(_) => {
            println!(
                "copying from: {} {}, to: {} {}",
                &source_path.exists(),
                &source_path.display(),
                &target_path.exists(),
                &target_path.display()
            );
            fs::copy(source_path, target_path)?;
        }
    }

    Ok(())
}
