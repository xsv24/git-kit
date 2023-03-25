use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

// Implement our own try_into just as a work around to implement
// TryInto for external types outside our crate.
pub trait TryConvert<T> {
    fn try_convert(self) -> anyhow::Result<T>;
}


impl TryConvert<String> for PathBuf {
    fn try_convert(self) -> anyhow::Result<String> {
        let path = self
            .to_str()
            .context("Failed to convert path into string")?;

        Ok(path.into())
    }
}

impl TryConvert<String> for &PathBuf {
    fn try_convert(self) -> anyhow::Result<String> {
        self.to_owned().try_convert()
    }
}

impl TryConvert<PathBuf> for String {
    fn try_convert(self) -> anyhow::Result<PathBuf> {
        let path = Path::new(&self);
        let absolute_path =
            fs::canonicalize(path).context("Failed to convert to an absolute path")?;

        if absolute_path.exists() {
            Ok(absolute_path)
        } else {
            Err(anyhow::anyhow!("Expected file not found."))
        }
    }
}

impl TryConvert<PathBuf> for &String {
    fn try_convert(self) -> anyhow::Result<PathBuf> {
        self.to_owned().try_convert()
    }
}
