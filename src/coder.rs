//! Minimal Coder REST client (direct HTTP — no dependency on the main repo).
//!
//! Implements exactly the endpoints `tenant add` needs: session user, GitHub
//! external-auth status, workspace creation, and readiness polling.

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct CoderUser {
    #[serde(default, alias = "user_id")]
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalAuth {
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub has_refresh_token: bool,
    #[serde(default)]
    pub app_install_url: String,
    #[serde(default)]
    pub user: Option<ExternalAuthUser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalAuthUser {
    #[serde(default)]
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalAuthDevice {
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub expires_in: u64,
    #[serde(default)]
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoderTemplate {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoderAgent {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub lifecycle_state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoderResource {
    #[serde(default)]
    pub agents: Vec<CoderAgent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoderBuild {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub resources: Vec<CoderResource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoderWorkspace {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub owner_name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub latest_build: Option<CoderBuild>,
}

#[derive(Debug, Clone)]
pub struct CoderClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl CoderClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn session_token(&self) -> &str {
        &self.token
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .with_context(|| format!("failed to reach Coder at {}", self.base_url))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!(
                "Coder returned {status} for GET {path}: {}",
                resp.text()
                    .await
                    .unwrap_or_default()
                    .chars()
                    .take(200)
                    .collect::<String>()
            ));
        }
        resp.json::<T>()
            .await
            .context("failed to decode Coder response")
    }

    pub async fn get_me(&self) -> Result<CoderUser> {
        self.get_json("/api/v2/users/me").await
    }

    pub async fn get_external_auth(&self, id: &str) -> Result<ExternalAuth> {
        self.get_json(&format!("/api/v2/external-auth/{id}")).await
    }

    pub async fn get_external_auth_device(&self, id: &str) -> Result<ExternalAuthDevice> {
        self.get_json(&format!("/api/v2/external-auth/{id}/device"))
            .await
    }

    pub async fn list_templates(&self) -> Result<Vec<CoderTemplate>> {
        self.get_json("/api/v2/templates").await
    }

    pub async fn list_workspaces(&self) -> Result<Vec<CoderWorkspace>> {
        #[derive(Deserialize)]
        struct WorkspaceList {
            #[serde(default)]
            workspaces: Vec<CoderWorkspace>,
        }
        let list: WorkspaceList = self.get_json("/api/v2/workspaces").await?;
        Ok(list.workspaces)
    }

    pub async fn get_workspace(&self, id: &str) -> Result<CoderWorkspace> {
        self.get_json(&format!("/api/v2/workspaces/{id}")).await
    }

    /// Create a workspace from a template by name, passing rich parameters.
    /// On HTTP 409 (already exists) it returns the existing workspace.
    pub async fn create_workspace(
        &self,
        user_id: &str,
        template_name: &str,
        name: &str,
        parameters: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<CoderWorkspace> {
        let templates = self.list_templates().await?;
        let template = templates
            .iter()
            .find(|t| t.name == template_name)
            .with_context(|| format!("template '{template_name}' not found in Coder"))?;

        let rich_parameter_values: Vec<serde_json::Value> = parameters
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string().trim_matches('"').to_string(),
                };
                serde_json::json!({ "name": k, "value": value })
            })
            .collect();

        let body = serde_json::json!({
            "template_id": template.id,
            "name": name,
            "rich_parameter_values": rich_parameter_values,
        });

        let resp = self
            .http
            .post(format!(
                "{}/api/v2/users/{user_id}/workspaces",
                self.base_url
            ))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("failed to send workspace creation request")?;
        let status = resp.status();

        if status == reqwest::StatusCode::CONFLICT {
            // Idempotent: the workspace already exists; return it.
            let ws = self
                .list_workspaces()
                .await?
                .into_iter()
                .find(|w| w.name == name)
                .with_context(|| format!("workspace '{name}' reported existing but not found"))?;
            return Ok(ws);
        }
        if !status.is_success() {
            return Err(anyhow!(
                "Coder returned {status} creating workspace: {}",
                resp.text()
                    .await
                    .unwrap_or_default()
                    .chars()
                    .take(300)
                    .collect::<String>()
            ));
        }
        resp.json::<CoderWorkspace>()
            .await
            .context("failed to decode created workspace")
    }

    /// Poll a workspace until its agent is ready, or `timeout` elapses.
    pub async fn wait_for_workspace_ready(
        &self,
        workspace_id: &str,
        timeout: Duration,
    ) -> Result<()> {
        let start = std::time::Instant::now();
        loop {
            let ws = self.get_workspace(workspace_id).await?;
            if is_agent_ready(&ws) {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                anyhow::bail!(
                    "timed out waiting for workspace '{}' to be ready",
                    workspace_id
                );
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

fn is_agent_ready(ws: &CoderWorkspace) -> bool {
    let Some(build) = &ws.latest_build else {
        return false;
    };
    let Some(agent) = build.resources.iter().find_map(|r| r.agents.first()) else {
        return false;
    };
    // Mirrors the main repo's readiness check.
    let status_ok = matches!(agent.status.as_str(), "connected" | "timeout" | "unknown");
    if !status_ok {
        return false;
    }
    if agent.status == "connected" {
        return true;
    }
    agent.lifecycle_state == "ready" && ws.status == "running"
}
