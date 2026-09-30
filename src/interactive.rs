//! Interactive terminal prompts (this is a terminal CLI, not a web UI).
//!
//! The browser is only involved when the user must fetch a secret (the Coder
//! session token) and paste it back into the terminal.

use anyhow::Result;
use dialoguer::{Input, Password};

/// Prompt for the Coder server URL, defaulting to `default`.
pub fn prompt_coder_url(default: &str) -> Result<String> {
    let url: String = Input::new()
        .with_prompt("Coder server URL")
        .default(default.to_string())
        .interact_text()?;
    Ok(url.trim_end_matches('/').to_string())
}

/// Interactive token acquisition: instruct the user to create a Coder session
/// token in their browser, then paste it back.
pub fn prompt_coder_token(coder_url: &str) -> Result<String> {
    println!();
    println!("  ─── Coder session token ───");
    println!();
    println!("  To use this CLI you need a Coder session token:");
    println!();
    println!("  1. Open this page in your browser:");
    println!("     {}/settings/tokens", coder_url.trim_end_matches('/'));
    println!("  2. Click \"Create Token\", give it a name (e.g. openflows-cli), copy it.");
    println!("  3. Paste the token below.");
    println!();
    let token: String = Password::new()
        .with_prompt("Paste your Coder session token")
        .interact()?;
    Ok(token.trim().to_string())
}

/// Prompt to retry / abort when a step needs a decision.
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    Ok(dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()?)
}
