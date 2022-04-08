use color_eyre::eyre::{eyre, Result};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

mod context;
mod manifest;

pub use context::*;
pub use manifest::*;

/// The default table of the config
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub active_context: String,
    pub context: HashMap<String, Context>,
}

impl Default for Config {
    fn default() -> Self {
        let mut context = HashMap::new();
        context.insert("localhost".to_owned(), Context::default());

        Self {
            active_context: "localhost".to_owned(),
            context,
        }
    }
}

impl Config {
    pub fn load(path: PathBuf) -> Result<Self> {
        if path.exists() {
            Ok(toml_edit::de::from_str(&read_to_string(path)?)?)
        } else {
            Ok(Config::default())
        }
    }

    pub fn get_active_context(&self) -> Result<&Context> {
        let context = &self.active_context;

        self.context
            .get(context)
            .ok_or_else(|| eyre!("Non-existent context: {context}"))
    }
}
