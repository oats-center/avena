use clap::{Parser, Subcommand};
use color_eyre::Result;

use avena::Avena;
use comfy_table::{Attribute, Cell, Table};

#[derive(Debug, Parser)]
pub struct DeviceCommand {
    #[clap(subcommand)]
    command: DevicesCommands,
}

#[derive(Debug, Subcommand)]
pub enum DevicesCommands {
    /// List all nodes known to the active context
    Ls,

    /// Remove a node from the active context
    Rm,

    /// Add a node to the active context
    Add,

    /// Ping nodes in the active context
    Ping,
}

pub fn exec(a: Avena, nodes: DeviceCommand) -> Result<()> {
    match nodes.command {
        DevicesCommands::Ls => {
            let devices = a.get_devices();

            let mut table = Table::new();
            table
                .load_preset(comfy_table::presets::UTF8_FULL)
                .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").add_attribute(Attribute::Bold),
                    Cell::new("Version").add_attribute(Attribute::Bold),
                ]);

            for (name, device) in devices.iter() {
                table.add_row(vec![name, &device.version]);
            }

            println!("{table}");
        }
        DevicesCommands::Rm => todo!(),
        DevicesCommands::Add => todo!(),
        DevicesCommands::Ping => {
            println!("Publish Ping command");
            let r = a.ping("test123");

            println!("Recieved response: {:#?}", r);
        }
    };

    Ok(())
}
