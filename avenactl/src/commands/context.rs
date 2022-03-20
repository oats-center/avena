use clap::Parser;
use color_eyre::{eyre::eyre, Result};

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, Table,
};

use toml_edit::{value, Entry};

use crate::config::{Config, Context, Manifest};
use crate::CONFIG_PATH;

#[derive(Debug, Parser)]
pub struct ContextCommand {
    #[clap(subcommand)]
    command: ContextCommands,
}

#[derive(Debug, clap::Subcommand)]
enum ContextCommands {
    /// List
    Ls,

    /// Remove
    Rm {
        #[clap(required = true)]
        /// Name of context to remove from local configuration
        name: String,
    },

    /// Add
    Add {
        #[clap(required = true)]
        /// Name of context to add to local configuration
        name: String,

        /// NATS connection string
        connection: String,
    },
}

pub fn exec(cmd: ContextCommand) -> Result<()> {
    match cmd.command {
        ContextCommands::Ls => {
            let config = Config::load(CONFIG_PATH.to_path_buf())?;

            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").add_attribute(Attribute::Bold),
                    Cell::new("Connection String").add_attribute(Attribute::Bold),
                ]);

            for (name, context) in config.context.into_iter() {
                let cell_name = if name == config.active_context {
                    Cell::new(format!("{name}*"))
                        .fg(Color::Green)
                        .add_attribute(Attribute::Bold)
                } else {
                    Cell::new(name)
                };

                table.add_row(vec![cell_name, Cell::new(context.connection)]);
            }

            println!("{table}");
        }

        ContextCommands::Rm { name } => {
            let mut m = Manifest::open(CONFIG_PATH.to_path_buf())?;

            match m.get_section_mut("context").entry(&name) {
                Entry::Occupied(context) => context.remove(),
                Entry::Vacant(_) => return Err(eyre!("Context '{name}' not found.")),
            };

            m.save()?;
        }

        ContextCommands::Add { name, connection } => {
            let mut m = Manifest::open(CONFIG_PATH.to_path_buf())?;

            let context = m.get_section_mut("context");

            context.insert(&name, Context::new(&name, &connection).try_into()?);

            // If the next context is the only context, then make it active
            if context.len() == 1 {
                m.get_table_mut().insert("active_context", value(name));
            }

            m.save()?;
        }
    };

    Ok(())
}
