//! Config storage for the Coder URL and session token.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

/// Default config file: `~/.config/openflows-cli/config.toml`.
pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openflows-cli/config.toml")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Base URL of the Coder server.
    #[serde(default)]
    pub coder_url: Option<String>,
    /// Coder session token (the operator's own token).
    #[serde(default)]
    pub coder_token: Option<String>,
}

pub fn load(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
}

pub fn save(path: &PathBuf, config: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }
    let raw = toml::to_string_pretty(config)?;

    // Create owner-only (0600) — a umask can only remove bits, so this can
    // never leave the token world-readable.
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts
        .open(path)
        .with_context(|| format!("failed to open config {}", path.display()))?;
    file.write_all(raw.as_bytes())
        .with_context(|| format!("failed to write config {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush config {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .context("failed to set config permissions")?;
    }
    Ok(())
}
