use crate::{display, logs, misc, stats};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

static ANSI_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();

fn get_ansi_regex() -> &'static Regex {
    ANSI_ESCAPE_RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap())
}

pub async fn parse_line(line: &str, display: &mut display::DisplayManager) -> Result<()> {
    // Check if line starts with @nix or @cargo

    let json_content = if let Some(content) = line.strip_prefix("@nix ") {
        content
    } else if let Some(content) = line.strip_prefix("@cargo ") {
        content
    } else {
        // Not a JSON line, ignore
        return Ok(());
    };
    // Clean ANSI escape sequences
    let clean_content = get_ansi_regex().replace_all(json_content, "");

    // Parse as fuzzy JSON - no fixed structs but enforce "action" field
    let value: Value =
        serde_json::from_str(&clean_content).map_err(|e| anyhow!("JSON parse error: {}", e))?;

    let obj = value
        .as_object()
        .ok_or_else(|| anyhow!("Expected JSON object"))?;

    // Enforce that "action" field exists
    let action = obj
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'action' field"))?;
    //println!("{line}, {action}");

    // Get optional type field
    let message_type = obj.get("type").and_then(|v| v.as_u64());

    // Route based on action and type
    match (action, message_type) {
        // stats handling - type 104 starts, type 105 updates
        ("start", Some(104)) => {
            stats::handle_stats_start(obj, display).await?;
        }
        ("result", Some(105)) => {
            stats::handle_stats_update(obj, display).await?;
        }
        ("stop", _) => {
            // Check if this is a stats stop or log stop
            let id = obj.get("id").and_then(|v| v.as_u64());
            if let Some(id) = id {
                if stats::is_stats_id(id) {
                    stats::handle_stats_stop(obj, display).await?;
                } else if logs::has_log_buffer(id) {
                    logs::handle_log_stop(obj, display).await?;
                }
            }
        }

        // LOGGING handling - type 105 starts mkDerivation logs
        ("start", Some(105)) => {
            logs::handle_log_start(obj, display).await?;
        }
        ("result", Some(101)) => {
            logs::handle_log_line(obj, display).await?;
        }
        ("result", Some(104)) => {
            logs::handle_log_phase(obj, display).await?;
        }

        // MESSAGE handling
        ("msg", _) => {
            logs::handle_msg(obj, display).await?;
        }

        // Everything else goes to misc
        _ => {
            misc::handle_misc(obj, display).await?;
        }
    }

    Ok(())
}
