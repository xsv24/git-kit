use std::{fs::File, io::Read, path::PathBuf};

use super::TryConvert;

pub fn get_file_contents(path: &PathBuf) -> anyhow::Result<String> {
    let file_name = path.try_convert().unwrap_or_default();
    let mut buff = String::new();

    let mut reader = File::open(path).map_err(|e| {
        log::error!("Failed to open file at '{file_name}': {}", e);
        e
    })?;

    reader.read_to_string(&mut buff).map_err(|e| {
        log::error!("Failed to read file at '{file_name}': {}", e);
        e
    })?;

    Ok(buff)
}
