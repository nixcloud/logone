use crate::{display, logs, status, LogLevel};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

static ANSI_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();
static DERIVATION_FAILURES: OnceLock<Mutex<HashMap<u64, bool>>> = OnceLock::new();
static ACTIVE_DERIVATIONS: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();
static DERIVATION_NAMES: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();

fn get_ansi_regex() -> &'static Regex {
    ANSI_ESCAPE_RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap())
}

fn get_failures_map() -> &'static Mutex<HashMap<u64, bool>> {
    DERIVATION_FAILURES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_active_derivations() -> &'static Mutex<HashSet<u64>> {
    ACTIVE_DERIVATIONS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn get_derivation_names() -> &'static Mutex<HashMap<u64, String>> {
    DERIVATION_NAMES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn add_active_derivation(id: u64, name: &str) {
    if let Ok(mut active) = get_active_derivations().lock() {
        active.insert(id);
    }
    if let Ok(mut names) = get_derivation_names().lock() {
        names.insert(id, name.to_string());
    }
}

fn remove_active_derivation(id: u64) {
    if let Ok(mut active) = get_active_derivations().lock() {
        active.remove(&id);
    }
    if let Ok(mut names) = get_derivation_names().lock() {
        names.remove(&id);
    }
}

fn find_derivation_id_for_error(msg: &str) -> Option<u64> {
    // Use the same regex pattern as in logs.rs to extract derivation names from error messages
    let re = regex::Regex::new(r#"/nix/store/([a-zA-Z0-9_.+-]+)\.drv"#).ok()?;
    let captures = re.captures(msg)?;
    let capture = captures.get(1)?
        .as_str();
    
    // Build the derivation name in the same format as logs.rs
    let drv_name = format!("building '/nix/store/{}.drv'", capture);
    
    // Find the ID that matches this derivation name
    if let Ok(names) = get_derivation_names().lock() {
        for (id, name) in names.iter() {
            if name == &drv_name {
                return Some(*id);
            }
        }
    }
    
    None
}

fn mark_derivation_failed(id: u64) {
    if let Ok(mut failures) = get_failures_map().lock() {
        failures.insert(id, true);
    }
}

fn is_derivation_failed(id: u64) -> bool {
    if let Ok(failures) = get_failures_map().lock() {
        failures.get(&id).copied().unwrap_or(false)
    } else {
        false
    }
}

fn remove_derivation_tracking(id: u64) {
    if let Ok(mut failures) = get_failures_map().lock() {
        failures.remove(&id);
    }
}

fn check_stop_for_failure(obj: &Map<String, Value>) -> bool {
    // Check for exit codes - non-zero typically indicates failure
    if let Some(exit_code) = obj.get("exitCode").and_then(|v| v.as_i64()) {
        if exit_code != 0 {
            return true;
        }
    }
    
    // Check for result field that might indicate failure
    if let Some(result) = obj.get("result").and_then(|v| v.as_i64()) {
        if result != 0 {
            return true;
        }
    }
    
    // Check for failure status fields
    if let Some(status) = obj.get("status").and_then(|v| v.as_str()) {
        if status.contains("fail") || status.contains("error") {
            return true;
        }
    }
    
    // Check for error messages in the stop payload
    if let Some(msg) = obj.get("msg").and_then(|v| v.as_str()) {
        let is_error = msg.contains("error") || msg.contains("failed") || 
           msg.contains("Error") || msg.contains("Failed") || msg.contains("FAILED");
        if is_error {
            return true;
        }
    }
    
    false
}

pub async fn parse_nix_line(line: &str, display: &mut display::DisplayManager) -> Result<()> {
    let json_content = if let Some(content) = line.strip_prefix("@nix ") {
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

    let obj: &Map<String, Value> = value
        .as_object()
        .ok_or_else(|| anyhow!("Expected JSON object"))?;

    let action: &str = obj
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'action' field"))?;

    let message_type: Option<u64> = obj.get("type").and_then(|v| v.as_u64());
    // check for embedded @cargo log message
    if message_type == Some(101) && action == "result" {
        let fields = obj.get("fields").and_then(|v| v.as_array());

        let content = if let Some(fields) = fields {
            if !fields.is_empty() {
                fields[0].as_str().unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if content.starts_with("@cargo") {
            // Only process @cargo messages in "cargo" mode
            match display.level() {
                LogLevel::Cargo => {
                    let id = obj
                        .get("id")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| anyhow!("Missing id in log line"))?;
                    return crate::parse::parse_cargo_line(id, &content, display).await;
                }
                LogLevel::Errors | LogLevel::Verbose => {
                    // In "errors" and "verbose" modes, ignore @cargo messages
                    return Ok(());
                }
            }
        }
    }
    process_event(obj, action, message_type, display).await
}

pub async fn parse_cargo_line(
    id: u64,
    line: &str,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let json_content = if let Some(content) = line.strip_prefix("@cargo ") {
        content
    } else {
        return Ok(());
    };

    // Clean ANSI escape sequences
    let clean_content = get_ansi_regex().replace_all(json_content, "");

    // Parse as fuzzy JSON - no fixed structs but enforce "action" field
    let mut value: Value =
        serde_json::from_str(&clean_content).map_err(|e| anyhow!("JSON parse error: {}", e))?;

    let obj: &mut Map<String, Value> = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("Expected JSON object"))?;

    obj.insert("id".to_string(), id.into());
    // Get optional type field
    let message_type: Option<u64> = obj.get("type").and_then(|v| v.as_u64());
    process_event(obj, "cargo", message_type, display).await
}

pub async fn process_event(
    obj: &Map<String, Value>,
    action: &str,
    message_type: Option<u64>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    // Apply filtering based on log level
    let log_level = display.level();
    
    // Route based on action and type
    match (action, message_type) {
        // STATUS handling - type 104 starts, type 105 updates
        ("start", Some(104)) => {
            status::handle_status_start(obj, display).await?;
        }
        ("result", Some(105)) => {
            status::handle_status_update(obj, display).await?;
        }
        ("stop", _) => {
            // Check if this is a status stop or log stop
            let id = obj.get("id").and_then(|v| v.as_u64());
            if let Some(id) = id {
                if status::is_status_id(id) {
                    status::handle_status_stop(obj, display).await?;
                } else if logs::has_log_buffer(id, display) {
                    // Handle log stop based on log level and failure status
                    match log_level {
                        LogLevel::Errors => {
                            // Check if the stop payload itself indicates failure
                            let stop_indicates_failure = check_stop_for_failure(obj);
                            if stop_indicates_failure {
                                mark_derivation_failed(id);
                            }
                            
                            // In errors mode, only flush logs if the derivation failed
                            if is_derivation_failed(id) {
                                logs::handle_log_stop(obj, display).await?;
                            }
                            // For non-failed builds in errors mode, we skip handle_log_stop
                            // which effectively drops the buffer without printing
                            
                            // Clean up tracking
                            remove_derivation_tracking(id);
                            remove_active_derivation(id);
                        }
                        LogLevel::Verbose => {
                            // In verbose mode, always flush all logs
                            logs::handle_log_stop(obj, display).await?;
                            remove_active_derivation(id);
                        }
                        LogLevel::Cargo => {
                            // In cargo mode, @nix logs are ignored anyway
                            // Clean up any failure tracking
                            remove_derivation_tracking(id);
                            remove_active_derivation(id);
                        }
                    }
                }
            }
        }

        // LOGGING handling - type 105 starts mkDerivation logs
        ("start", Some(105)) => {
            // Only process @nix logs in "errors" and "verbose" modes
            match log_level {
                LogLevel::Errors | LogLevel::Verbose => {
                    logs::handle_log_start(obj, display).await?;
                    
                    // Track active derivation for proper failure attribution
                    if let Some(id) = obj.get("id").and_then(|v| v.as_u64()) {
                        let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        add_active_derivation(id, text);
                    }
                }
                LogLevel::Cargo => {
                    // In "cargo" mode, ignore @nix logs
                }
            }
        }
        ("result", Some(101)) => {
            // Only process @nix log lines in "errors" and "verbose" modes
            match log_level {
                LogLevel::Errors | LogLevel::Verbose => {
                    logs::handle_log_line(obj, display).await?;
                }
                LogLevel::Cargo => {
                    // In "cargo" mode, ignore @nix log lines
                }
            }
        }
        ("result", Some(104)) => {
            // Only process @nix log phases in "errors" and "verbose" modes
            match log_level {
                LogLevel::Errors | LogLevel::Verbose => {
                    logs::handle_log_phase(obj, display).await?;
                }
                LogLevel::Cargo => {
                    // In "cargo" mode, ignore @nix log phases
                }
            }
        }

        // MESSAGE handling
        ("msg", _) => {
            // Process messages differently based on log level
            match log_level {
                LogLevel::Cargo => {
                    // In "cargo" mode, suppress ALL @nix messages
                    // Only @cargo messages should be processed in cargo mode
                }
                LogLevel::Errors => {
                    let msg = obj.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                    let level = obj.get("level").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    // Check if this message indicates a build failure
                    let is_error = level >= 3 || msg.contains("error") || msg.contains("failed") || 
                       msg.contains("Error") || msg.contains("Failed") || msg.contains("FAILED") ||
                       msg.contains("cannot") || msg.contains("Could not");
                    
                    if is_error {
                        // Try to find the specific derivation this error belongs to
                        if let Some(failing_id) = find_derivation_id_for_error(msg) {
                            // Mark only the specific failing derivation, not all active ones
                            mark_derivation_failed(failing_id);
                        }
                        // Conservative approach: if we can't definitively attribute the error
                        // to a specific derivation, don't mark any as failed rather than
                        // incorrectly marking all active derivations as failed
                        
                        logs::handle_msg(obj, display).await?;
                    }
                }
                LogLevel::Verbose => {
                    // In "verbose" mode, handle all @nix messages
                    logs::handle_msg(obj, display).await?;
                }
            }
        }

        // CARGO message handling
        ("cargo", Some(0)) => {
            // Only process @cargo messages in "cargo" mode
            match log_level {
                LogLevel::Cargo => {
                    logs::handle_cargo_log_start(obj, display).await?;
                }
                LogLevel::Errors | LogLevel::Verbose => {
                    // In "errors" and "verbose" modes, ignore @cargo messages
                }
            }
        }
        ("cargo", Some(2)) => {
            // Only process @cargo messages in "cargo" mode
            match log_level {
                LogLevel::Cargo => {
                    logs::handle_cargo_log_exit(obj, display).await?;
                }
                LogLevel::Errors | LogLevel::Verbose => {
                    // In "errors" and "verbose" modes, ignore @cargo messages
                }
            }
        }

        _ => {}
    }

    Ok(())
}
