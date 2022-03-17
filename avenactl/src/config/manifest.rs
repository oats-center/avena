use color_eyre::eyre::{eyre, Context, Result};
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::PathBuf,
};
use toml_edit::{table, Document, Table};

use super::Config;

/// A wraper around `toml_edit` to factor out oft repeated code.
pub struct Manifest {
    path: PathBuf,
    doc: Document,
}

impl Manifest {
    pub fn open(path: PathBuf) -> Result<Self> {
        let doc = read_to_string(&path)?.parse::<Document>()?;

        Ok(Self { path, doc })
    }

    pub fn save(&self) -> Result<()> {
        if !self.path.exists() {
            create_dir_all(
                self.path
                    .parent()
                    .ok_or_else(|| eyre!("{path} has no parent?", path = self.path.display()))?,
            )?;
        }

        write(&self.path, self.doc.to_string())?;

        Ok(())
    }

    pub fn get_table_mut(&mut self, name: &str) -> &mut Table {
        self.doc
            .as_table_mut()
            .entry(name)
            .or_insert_with(table)
            .as_table_mut()
            .unwrap()
    }
}

impl TryInto<Config> for Manifest {
    type Error = color_eyre::Report;

    fn try_into(self) -> Result<Config> {
        toml_edit::de::from_document(self.doc).wrap_err("Invalid config file.")
    }
}
