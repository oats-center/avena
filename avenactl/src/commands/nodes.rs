use clap::{Parser, Subcommand};
use color_eyre::Result;

#[derive(Debug, Parser)]
pub struct NodesCommand {
    #[clap(subcommand)]
    command: NodesCommands,
}

#[derive(Debug, Subcommand)]
pub enum NodesCommands {
    /// List all nodes known to the active context
    Ls,

    /// Remove a node from the active context
    Rm,

    /// Add a node to the active context
    Add,

    /// Ping nodes in the active context
    Ping,
}

pub fn exec(nodes: NodesCommand) -> Result<()> {
    todo!();
}
