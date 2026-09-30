//! OpenFlows CLI — a self-contained terminal CLI that onboards a tenant to a
//! deployed OpenFlows instance.
//!
//! It authenticates **directly to Coder** using a Coder session token (collected
//! interactively), and performs the whole `tenant add` provisioning itself — it
//! does not depend on the main `openflows` binary or the manager API.

pub mod coder;
pub mod config;
pub mod interactive;
pub mod registry;
pub mod tenant;

pub use clap::Parser;

/// Coder URL used for the demo deployment. This is hardcoded for now and should
/// be changed to the real deployment URL, then the package re-published.
pub const DEFAULT_CODER_URL: &str = "https://coder.test.com";
