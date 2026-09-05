//! Registering Sheetz with MCP clients.
//!
//! A listening socket is useless if the assistant does not know Sheetz exists,
//! and registering a server normally means hand-editing a client's JSON config
//! — exactly the terminal step this is meant to avoid. So Sheetz writes the
//! entry itself, from the installer or from the File view.
//!
//! Existing config is merged, never replaced, and a `.bak` is left behind.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// A client config we know how to write, and whether it is present.
pub struct Client {
    pub name: &'static str,
    pub path: PathBuf,
    pub registered: bool,
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Config files of the MCP clients we support, whether or not they exist yet.
pub fn known_clients() -> Vec<Client> {
    let Some(home) = home() else {
        return Vec::new();
    };
    let candidates: Vec<(&'static str, PathBuf)> = vec![
        (
            "Claude Desktop",
            home.join(".config/Claude/claude_desktop_config.json"),
        ),
        ("Claude Code", home.join(".claude.json")),
    ];
    candidates
        .into_iter()
        .map(|(name, path)| {
            let registered = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                .map(|v| v.pointer("/mcpServers/sheetz").is_some())
                .unwrap_or(false);
            Client {
                name,
                path,
                registered,
            }
        })
        .collect()
}

/// The command an MCP client should run to reach this install.
fn server_entry() -> Result<Value> {
    let exe = std::env::current_exe().context("cannot find the sheetz binary")?;
    Ok(json!({
        "command": exe.display().to_string(),
        "args": ["mcp"],
    }))
}

/// Adds (or refreshes) the `sheetz` entry in one client config.
///
/// Only the one key is touched: everything else in the file is preserved, and
/// the previous contents are kept as `<file>.bak`.
pub fn register_one(path: &PathBuf) -> Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut root: Value = if existing.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(&existing)
            .with_context(|| format!("{} is not valid JSON", path.display()))?
    };
    if !existing.trim().is_empty() {
        let _ = std::fs::write(path.with_extension("json.bak"), &existing);
    }
    if !root.is_object() {
        root = json!({});
    }
    let servers = root
        .as_object_mut()
        .expect("object")
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers
        .as_object_mut()
        .expect("object")
        .insert("sheetz".to_string(), server_entry()?);

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    std::fs::write(path, serde_json::to_string_pretty(&root)?)
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(())
}

/// Registers with every client config that already exists.
///
/// Returns what was written and what was skipped, so the caller can tell the
/// user rather than claiming success silently.
pub fn register_all() -> (Vec<String>, Vec<String>) {
    let mut done = Vec::new();
    let mut skipped = Vec::new();
    for client in known_clients() {
        if !client.path.exists() {
            skipped.push(format!("{} (not installed)", client.name));
            continue;
        }
        match register_one(&client.path) {
            Ok(()) => done.push(client.name.to_string()),
            Err(e) => skipped.push(format!("{}: {e}", client.name)),
        }
    }
    (done, skipped)
}
