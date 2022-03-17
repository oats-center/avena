pub mod context;

use clap::Subcommand;

use context::ContextCommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage NATS contexts
    Context(ContextCommand),
}
