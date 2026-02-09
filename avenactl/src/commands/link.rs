use clap::{Parser, Subcommand};
use color_eyre::Result;

use avena::Avena;

#[derive(Debug, Parser)]
pub struct LinkCommand {
    #[clap(subcommand)]
    command: LinkCommands,
}

#[derive(Debug, Subcommand)]
pub enum LinkCommands {
    /// Add a link from one device to another's NATS
    Add {
        /// Source device that will connect outbound
        #[clap(long)]
        from: String,
        /// Target NATS URL or host
        #[clap(long)]
        to: String,
    },

    /// Remove a link from a device
    Rm {
        /// Source device
        #[clap(long)]
        from: String,
        /// Target NATS URL to unlink
        #[clap(long)]
        to: String,
    },

    /// List links for a device
    Ls {
        /// Device to query (optional, lists all if omitted)
        #[clap(long)]
        device: Option<String>,
    },
}

pub async fn exec(a: Avena, cmd: LinkCommand) -> Result<()> {
    match cmd.command {
        LinkCommands::Add { from, to } => {
            let remote_url = if to.contains("://") {
                to
            } else {
                format!("nats://{to}")
            };

            let resp = a.register_link(&from, &remote_url)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

            if resp.ok {
                println!("Link added: {} -> {}", from, remote_url);
            } else {
                println!("Failed to add link: {}", resp.message);
            }
        }
        LinkCommands::Rm { from, to } => {
            let remote_url = if to.contains("://") {
                to
            } else {
                format!("nats://{to}")
            };

            let resp = a.unregister_link(&from, &remote_url)
                .await
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

            if resp.ok {
                println!("Link removed: {} -> {}", from, remote_url);
            } else {
                println!("Failed to remove link: {}", resp.message);
            }
        }
        LinkCommands::Ls { device } => {
            match device {
                Some(dev) => {
                    println!("Links for device {} (not yet implemented - requires KV query)", dev);
                }
                None => {
                    println!("All links (not yet implemented - requires KV scan)");
                }
            }
        }
    }

    Ok(())
}
