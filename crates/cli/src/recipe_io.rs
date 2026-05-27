//! Recipe file I/O — TOML and JSON.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use django_bakery_engine::Recipe;

pub fn load(path: &Path) -> Result<Recipe> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading recipe file {}", path.display()))?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "toml" => toml::from_str::<Recipe>(&raw).context("parsing TOML recipe"),
        "json" => serde_json::from_str::<Recipe>(&raw).context("parsing JSON recipe"),
        other => bail!("unknown recipe format: .{other} (expected .toml or .json)"),
    }
}
