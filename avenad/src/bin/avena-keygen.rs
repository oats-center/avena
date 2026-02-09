use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use nkeys::KeyPair;
use tokio::fs;

mod nats_jwt_inner {
    pub use avenad::nats_jwt::*;
}
use nats_jwt_inner::{NatsJwtManager, setup_operator_mode};

#[derive(Parser)]
#[command(name = "avena-keygen")]
#[command(about = "Generate NATS JWT credentials for avena")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate full operator setup (operator, SYS, AVENA accounts, admin users)
    Init {
        /// Output directory for credentials
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Generate a user credential for leaf node connection
    LeafUser {
        /// Directory containing the account seed (AVENA.nk)
        #[arg(short, long)]
        account_dir: PathBuf,
        /// Name for the user
        #[arg(short, long)]
        name: String,
        /// Output path for credentials file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate a NATS server config for hub mode
    HubConfig {
        /// Directory containing the JWTs and seeds
        #[arg(short, long)]
        creds_dir: PathBuf,
        /// Leaf node listener port
        #[arg(long, default_value = "7422")]
        leaf_port: u16,
        /// Client port
        #[arg(long, default_value = "4222")]
        client_port: u16,
        /// Output path for config file
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { output } => {
            cmd_init(&output).await?;
        }
        Commands::LeafUser {
            account_dir,
            name,
            output,
        } => {
            cmd_leaf_user(&account_dir, &name, &output).await?;
        }
        Commands::HubConfig {
            creds_dir,
            leaf_port,
            client_port,
            output,
        } => {
            cmd_hub_config(&creds_dir, leaf_port, client_port, &output).await?;
        }
    }

    Ok(())
}

async fn cmd_init(output: &PathBuf) -> Result<()> {
    fs::create_dir_all(output).await?;
    setup_operator_mode(output).await?;
    println!("Generated credentials in {}", output.display());
    println!("Files created:");
    println!("  operator.nk    - Operator seed");
    println!("  operator.jwt   - Operator JWT");
    println!("  SYS.nk         - System account seed");
    println!("  SYS.jwt        - System account JWT");
    println!("  sys-admin.creds - System admin user credentials");
    println!("  AVENA.nk       - Avena account seed");
    println!("  AVENA.jwt      - Avena account JWT");
    println!("  avena-admin.creds - Avena admin user credentials");
    Ok(())
}

async fn cmd_leaf_user(account_dir: &PathBuf, name: &str, output: &PathBuf) -> Result<()> {
    let operator_seed = fs::read_to_string(account_dir.join("operator.nk")).await?;
    let operator_kp = KeyPair::from_seed(operator_seed.trim())?;
    let mgr = NatsJwtManager::from_keypair(operator_kp);

    let avena_seed = fs::read_to_string(account_dir.join("AVENA.nk")).await?;
    let avena_kp = KeyPair::from_seed(avena_seed.trim())?;

    let (jwt, user_kp) = mgr.generate_user_jwt(
        &avena_kp,
        name,
        vec![">".to_string()],
        vec![">".to_string()],
    )?;

    let creds = NatsJwtManager::create_creds_file(&jwt, &user_kp)?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(output, &creds).await?;

    println!("Generated leaf user credentials: {}", output.display());
    Ok(())
}

async fn cmd_hub_config(
    creds_dir: &PathBuf,
    leaf_port: u16,
    client_port: u16,
    output: &PathBuf,
) -> Result<()> {
    let operator_jwt = fs::read_to_string(creds_dir.join("operator.jwt")).await?;
    let sys_jwt = fs::read_to_string(creds_dir.join("SYS.jwt")).await?;
    let avena_jwt = fs::read_to_string(creds_dir.join("AVENA.jwt")).await?;

    let sys_seed = fs::read_to_string(creds_dir.join("SYS.nk")).await?;
    let sys_kp = KeyPair::from_seed(sys_seed.trim())?;
    let avena_seed = fs::read_to_string(creds_dir.join("AVENA.nk")).await?;
    let avena_kp = KeyPair::from_seed(avena_seed.trim())?;

    let config = format!(
        r#"server_name: avena-hub
port: {client_port}

jetstream {{
  store_dir: /data/jetstream
  domain: avena
}}

operator: {operator_jwt}
system_account: {sys_pub}

resolver: MEMORY
resolver_preload: {{
  {sys_pub}: {sys_jwt},
  {avena_pub}: {avena_jwt}
}}

leafnodes {{
  port: {leaf_port}
}}
"#,
        client_port = client_port,
        leaf_port = leaf_port,
        operator_jwt = operator_jwt.trim(),
        sys_pub = sys_kp.public_key(),
        sys_jwt = sys_jwt.trim(),
        avena_pub = avena_kp.public_key(),
        avena_jwt = avena_jwt.trim(),
    );

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(output, &config).await?;

    println!("Generated hub config: {}", output.display());
    Ok(())
}
