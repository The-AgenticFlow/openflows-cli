//! The `tenant add` provisioning flow, self-contained.

use crate::{
    coder::CoderClient,
    config::{self, Config},
    interactive,
    registry::Registry,
};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::time::Duration;

const TEMPLATE_NAME: &str = "openflows-nexus";
const EXTERNAL_AUTH_DEFAULT: &str = "primary-github";
const GITHUB_LINK_TIMEOUT: Duration = Duration::from_secs(300);
const READY_TIMEOUT: Duration = Duration::from_secs(300);

/// Validate a tenant name (must be safe for Redis namespaces / identifiers).
fn validate_tenant_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("tenant name must not be empty");
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        bail!(
            "tenant name '{}' may only contain ASCII letters, numbers, '.', '_' and '-'",
            name
        );
    }
    Ok(())
}

/// Resolve the Coder URL: flag > env > saved config > hardcoded default.
fn resolve_url(flag: Option<&str>, saved: &Config) -> String {
    flag.map(str::to_string)
        .or_else(|| std::env::var("CODER_URL").ok())
        .or_else(|| saved.coder_url.clone())
        .unwrap_or_else(|| crate::DEFAULT_CODER_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Resolve the Coder session token: flag > env > saved config > interactive.
async fn resolve_token(flag: Option<&str>, saved: &Config, coder_url: &str) -> Result<String> {
    if let Some(t) = flag {
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Ok(t) = std::env::var("CODER_SESSION_TOKEN") {
        if !t.trim().is_empty() {
            return Ok(t);
        }
    }
    if let Some(t) = &saved.coder_token {
        if !t.is_empty() {
            return Ok(t.clone());
        }
    }
    // Interactive: tell the user to create a token in the browser, paste it back.
    interactive::prompt_coder_token(coder_url)
}

fn resolve_hook_secret() -> Result<String> {
    if let Ok(s) = std::env::var("CODER_CHAT_HOOK_SECRET") {
        if !s.trim().is_empty() {
            return Ok(s);
        }
    }
    // For the demo, generate one and tell the operator to keep it consistent
    // with the controller. 32 random bytes, hex-encoded.
    let secret = random_hex(32);
    println!();
    println!("  ⚠ No CODER_CHAT_HOOK_SECRET set — generated one for this run:");
    println!("    {secret}");
    println!("    (Keep this consistent with the deployed controller / .env if needed.)");
    println!();
    Ok(secret)
}

fn resolve_hook_url() -> String {
    if let Ok(u) = std::env::var("CODER_CHAT_HOOK_URL") {
        if !u.trim().is_empty() {
            return u;
        }
    }
    if let Ok(u) = std::env::var("OPENFLOWS_HOOK_URL") {
        if !u.trim().is_empty() {
            return u;
        }
    }
    // Default matches the bundled stack.
    "http://openflows-nexus:3001/experimental/hooks/chat".to_string()
}

fn random_hex(bytes: usize) -> String {
    use std::io::Read;
    let mut data = vec![0u8; bytes];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut data);
    }
    const HEX: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes * 2);
    for b in data {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Run `tenant add`.
pub async fn run(
    repo: &str,
    name: Option<&str>,
    fleet: u32,
    url_flag: Option<&str>,
    token_flag: Option<&str>,
    config_path: &PathBuf,
) -> Result<()> {
    if fleet < 1 {
        bail!("--fleet must be >= 1 (a fleet of N means N FORGE-SENTINEL pairs); got {fleet}");
    }
    if !repo.contains('/') {
        bail!("repo must be in 'owner/repo' format; got '{repo}'");
    }
    let tenant_name = name
        .map(str::to_string)
        .unwrap_or_else(|| repo.split('/').next().unwrap_or(repo).to_string());
    validate_tenant_name(&tenant_name)?;

    // Resolve credentials.
    let saved = config::load(config_path)?;
    let coder_url = resolve_url(url_flag, &saved);
    let token = resolve_token(token_flag, &saved, &coder_url).await?;

    // Persist them for future runs.
    let updated = Config {
        coder_url: Some(coder_url.clone()),
        coder_token: Some(token.clone()),
    };
    config::save(config_path, &updated)?;

    // Build the fleet registry from the bundled base.
    let registry = Registry::bundled()?.with_team_fleet(fleet);
    let registry_json = registry.to_pretty_json()?;

    let hook_secret = resolve_hook_secret()?;
    let hook_url = resolve_hook_url();

    println!();
    println!("═══════════════════════════════════════");
    println!("  OpenFlows: Adding Tenant");
    println!("═══════════════════════════════════════");
    println!();
    println!("  Repo:          {repo}");
    println!("  Tenant:        {tenant_name}");
    println!("  Fleet:         {fleet} (FORGE-SENTINEL pair(s))");
    println!("  Coder:         {coder_url}");
    println!();

    let client = CoderClient::new(coder_url.clone(), token);

    // 1. Session user.
    let me = client
        .get_me()
        .await
        .context("failed to authenticate with Coder — is the session token valid?")?;
    println!("  ✓ Authenticated as '{}'", me.username);

    // 2. Wait for the GitHub external-auth link (self-serve device flow).
    let external_auth_id = std::env::var("CODER_EXTERNAL_AUTH_0_ID")
        .unwrap_or_else(|_| EXTERNAL_AUTH_DEFAULT.to_string());
    wait_for_github_link(&client, &coder_url, &external_auth_id).await?;

    // 3. Build the nexus workspace parameters.
    let repo_url = format!("https://github.com/{repo}.git");
    let nexus_name = format!("openflows-nexus-{tenant_name}");
    let mut params = serde_json::Map::new();
    params.insert("repo_url".into(), serde_json::json!(repo_url));
    params.insert("redis_url".into(), serde_json::json!("redis://redis:6379"));
    params.insert("coder_url".into(), serde_json::json!(coder_url));
    params.insert(
        "coder_session_token".into(),
        serde_json::json!(client.session_token()),
    );
    params.insert("tenant".into(), serde_json::json!(tenant_name));
    params.insert("github_repository".into(), serde_json::json!(repo));
    params.insert(
        "coder_chat_hook_secret".into(),
        serde_json::json!(hook_secret),
    );
    params.insert("coder_chat_hook_url".into(), serde_json::json!(hook_url));
    params.insert("registry_json".into(), serde_json::json!(registry_json));
    params.insert("start_controller".into(), serde_json::json!(true));

    // 4. Create (or reuse) the workspace.
    let ws = client
        .create_workspace(&me.id, TEMPLATE_NAME, &nexus_name, &params)
        .await
        .context("failed to create tenant nexus workspace")?;
    println!("  ✓ Nexus workspace '{}' ({})", ws.name, ws.id);

    // 5. Wait until the workspace agent is ready.
    println!("  Waiting for the workspace to be ready (up to 5 min)...");
    client
        .wait_for_workspace_ready(&ws.id, READY_TIMEOUT)
        .await
        .context("workspace did not become ready in time")?;

    println!();
    println!("  ✓ Tenant '{}' added", tenant_name);
    println!("  ✓ Fleet: {} FORGE-SENTINEL pair(s)", fleet);
    println!("  ✓ Nexus workspace: {}", nexus_name);
    println!();
    println!("  Create a GitHub issue in {repo} — OpenFlows will pick it up.");
    Ok(())
}

/// Poll the GitHub external-auth grant until linked, showing device flow info.
async fn wait_for_github_link(
    client: &CoderClient,
    coder_url: &str,
    external_auth_id: &str,
) -> Result<()> {
    println!();
    println!("  ─── GitHub Link Required ───");
    println!();
    let start = std::time::Instant::now();
    let mut link_shown = false;
    loop {
        match client.get_external_auth(external_auth_id).await {
            Ok(auth) if auth.authenticated => {
                let linked = auth
                    .user
                    .as_ref()
                    .map(|u| u.login.as_str())
                    .unwrap_or("(unknown)");
                println!("  ✓ GitHub link confirmed for '{}'", linked);
                return Ok(());
            }
            Ok(auth) => {
                if !link_shown {
                    if !auth.app_install_url.is_empty() {
                        println!(
                            "  1. Install/grant the OpenFlows GitHub App: {}",
                            auth.app_install_url
                        );
                    }
                    match client.get_external_auth_device(external_auth_id).await {
                        Ok(device) => {
                            let target = if !device.verification_uri_complete.is_empty() {
                                device.verification_uri_complete
                            } else {
                                device.verification_uri
                            };
                            if !target.is_empty() {
                                println!("  2. Authorize at: {}", target);
                            }
                            if !device.user_code.is_empty() {
                                println!("     and enter code: {}", device.user_code);
                            }
                        }
                        Err(_) => {
                            println!(
                                "  2. Complete the link in the dashboard: {}/external-auth/{}",
                                coder_url.trim_end_matches('/'),
                                external_auth_id
                            );
                        }
                    }
                    println!();
                    println!("  Waiting for the GitHub link to complete (up to 5 minutes)...");
                    link_shown = true;
                }
            }
            Err(e) => {
                eprintln!("  (could not query GitHub link status: {e}; retrying)");
            }
        }
        if start.elapsed() >= GITHUB_LINK_TIMEOUT {
            bail!(
                "timed out waiting for the GitHub link. Complete it at {}/external-auth/{} and rerun `tenant add`.",
                coder_url.trim_end_matches('/'),
                external_auth_id
            );
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
