//! Fleet registry handling: load the bundled base registry and apply the fleet
//! size to the FORGE/SENTINEL entries.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const BASE_REGISTRY: &str = include_str!("../registry.json");

/// Registry entry shape (the subset we need to apply the fleet and re-serialize).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub plan_mode: bool,
    #[serde(default = "default_max_instances")]
    pub max_instances: u32,
    #[serde(default)]
    pub instances: u32,
    #[serde(default)]
    pub cli: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub mcp: serde_json::Value,
}

fn default_max_instances() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default = "default_cli")]
    pub default_cli: String,
    #[serde(default = "default_allowed_domains")]
    pub allowed_domains: Vec<String>,
    pub team: Vec<RegistryEntry>,
}

fn default_cli() -> String {
    "claude".to_string()
}

fn default_allowed_domains() -> Vec<String> {
    vec!["api.github.com".to_string(), "*.github.com".to_string()]
}

impl Registry {
    /// Load the bundled base registry.
    pub fn bundled() -> Result<Self> {
        serde_json::from_str(BASE_REGISTRY).context("failed to parse bundled registry.json")
    }

    /// Apply the fleet size: set FORGE and SENTINEL max_instances (and the v1
    /// `instances` field) to `pairs`, preserving all other entries/fields.
    pub fn with_team_fleet(&self, pairs: u32) -> Registry {
        let mut clone = self.clone();
        for entry in clone.team.iter_mut() {
            if entry.id == "forge" || entry.id == "sentinel" {
                entry.max_instances = pairs;
                entry.instances = pairs;
            }
        }
        clone
    }

    /// Pretty-print as a JSON string (the value passed to the workspace as
    /// `registry_json`).
    pub fn to_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("failed to serialize registry")
    }
}
