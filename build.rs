use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

// Build script to copy over default templates & config into the binary directory.
fn main() {
    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "git-kit") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let package_version = env!("CARGO_PKG_VERSION");
        let config_dir = dirs.config_dir();

        match package_version {
            version if version <= "0.0.8" => {
                println!("Updating template files...");
                // We've moved away from separate template files in favor of a single yaml file.
                copy_or_replace(
                    &project_root.join("templates/"),
                    &config_dir.join("templates/"),
                )
                .expect("Failed to copy or update to the latest template files for git-kit")
            }
            _ => {
                println!("Updating config file...");
                copy_or_replace(
                    &project_root.join(".git-kit.yml"),
                    &config_dir.join(".git-kit.yml"),
                )
                .expect("Failed to copy or update to the latest config file for git-kit")
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
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
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}
