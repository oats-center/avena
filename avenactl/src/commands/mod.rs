pub mod context;
pub mod nodes;

use clap::Subcommand;

use context::ContextCommand;
use nodes::NodesCommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage NATS contexts
    Context(ContextCommand),
    Nodes(NodesCommand),
}
