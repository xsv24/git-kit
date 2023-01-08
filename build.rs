// Imported manually overwise we need a separate sub crate thats published.
#[path = "src/migrations.rs"]
mod migrations;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context;
use directories::ProjectDirs;
use migrations::{db_migrations, MigrationContext};
use rusqlite::Connection;

// Build script to copy over default templates & config into the binary directory.
fn main() -> anyhow::Result<()> {
    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "git-kit") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config_dir = dirs.config_dir();

        println!("Updating config file...");

        // Create config dir if not exists.
        fs::create_dir(config_dir).ok();

        copy_or_replace(&project_root.join("templates"), &config_dir.to_path_buf())
            .context("Failed to copy or update to the latest config file for git-kit")?;

        let mut connection =
            Connection::open(config_dir.join("db")).expect("Failed to open sqlite connection");

        db_migrations(
            &mut connection,
            MigrationContext {
                default_configs: Some(migrations::DefaultConfig {
                    default: config_dir.join("default.yml"),
                    conventional: config_dir.join("conventional.yml"),
                }),
                version: Some(4),
            },
        )
        .expect("Failed to apply migrations.");
        connection
            .close()
            .expect("Failed to close sqlite connection");
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn copy_or_replace(source_path: &PathBuf, target_path: &PathBuf) -> anyhow::Result<()> {
    match fs::read_dir(source_path) {
        Ok(entry_iter) => {
            fs::create_dir_all(target_path)
                .with_context(|| format!("Failed to create dir {}", target_path.display()))?;

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
            fs::copy(source_path, target_path)
                .with_context(|| format!("Failed to copy from: {:?}, to: {:?}", source_path.as_os_str(), target_path.as_os_str()))?;
        }
    }

    Ok(())
}
