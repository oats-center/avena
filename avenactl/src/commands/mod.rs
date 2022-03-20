pub mod context;
pub mod devices;

use clap::Subcommand;

use context::ContextCommand;
use devices::DeviceCommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage NATS contexts
    Context(ContextCommand),

    /// Manage Avena fleet devices
    Devices(DeviceCommand),
}
