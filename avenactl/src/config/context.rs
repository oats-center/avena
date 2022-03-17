use color_eyre::eyre::{self, eyre, Result};
use serde_derive::{Deserialize, Serialize};
use toml_edit::ser::to_item;
use toml_edit::Item;

/// The [context] table of the config
#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    pub name: String,
    pub connection: String,
}

impl Context {
    pub fn new(name: String, connection: String) -> Self {
        Self { name, connection }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            name: "localhost".to_owned(),
            connection: "localhost".to_owned(),
        }
    }
}

impl TryInto<Item> for Context {
    type Error = eyre::Report;

    fn try_into(self) -> Result<Item> {
        Ok(Item::Table(to_item(&self)?.into_table().map_err(|_| {
            eyre!("Context struct doesn't map to TOML table?")
        })?))
    }
}
