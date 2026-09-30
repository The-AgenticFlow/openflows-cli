use anyhow::Result;
use clap::{Parser, Subcommand};
use openflows_cli::config;

#[derive(Debug, Parser)]
#[command(
    name = "openflows-cli",
    about = "Self-contained terminal CLI to onboard a tenant to a deployed OpenFlows instance",
    version
)]
pub struct Cli {
    /// Coder server URL (overrides config / env).
    #[arg(long, global = true, env = "CODER_URL")]
    pub url: Option<String>,

    /// Coder session token (overrides config / env / interactive prompt).
    #[arg(long, global = true, env = "CODER_SESSION_TOKEN")]
    pub token: Option<String>,

    /// Path to the config file (defaults to ~/.config/openflows-cli/config.toml).
    #[arg(long, global = true)]
    pub config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Onboard a tenant bound to a GitHub repository.
    Tenant {
        #[command(subcommand)]
        action: TenantAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum TenantAction {
    /// Add a tenant bound to a GitHub repository.
    Add {
        /// Repository in `owner/repo` format.
        repo: String,
        /// Tenant name (defaults to the repo owner).
        #[arg(long)]
        name: Option<String>,
        /// Fleet size: number of FORGE-SENTINEL pairs (>= 1).
        #[arg(long)]
        fleet: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(config::default_config_path);

    match cli.command {
        Command::Tenant {
            action: TenantAction::Add { repo, name, fleet },
        } => {
            openflows_cli::tenant::run(
                &repo,
                name.as_deref(),
                fleet,
                cli.url.as_deref(),
                cli.token.as_deref(),
                &config_path,
            )
            .await
        }
    }
}
