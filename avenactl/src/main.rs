mod commands;
mod config;

use avena::Avena;
use clap::Parser;
use config::Config;
use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};
use directories::ProjectDirs;
use lazy_static::lazy_static;

use commands::Commands;

lazy_static! {
    pub static ref CONFIG_PATH: PathBuf = ProjectDirs::from("org", "oatscenter", "avena")
        .ok_or_else(|| eyre!("Can not compute project config path"))
        .unwrap()
        .config_dir()
        .join("config.toml");
}

#[derive(Parser, Debug)]
#[clap(name = "avenactl")]
#[clap(author, version, about, long_about = None)]
/// Manage Avena based devices
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    // Use color_eyre for applcation error handling
    color_eyre::install()?;

    // Parse CLI agruments
    let args = Cli::parse();

    // Load Config
    let config = Config::load(CONFIG_PATH.to_path_buf())?;

    // Connect to Avena context
    let a = Avena::connect(&config.get_active_context()?.connection);

    // Pass control the commanded subcommand
    match args.command {
        Commands::Context(context) => commands::context::exec(context),
        Commands::Devices(node) => commands::devices::exec(a, node),
    }?;

    Ok(())
}
