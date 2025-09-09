use crate::display;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone)]
pub struct NixMessage {
    pub action: String,
    pub message_type: Option<u64>,
    pub content: String,
    pub level: Option<u64>,
    pub file: Option<String>,
}

pub type LogData = Vec<NixMessage>;

// Global state for logs
static LOG_BUFFERS: LazyLock<Mutex<HashMap<u64, LogData>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static DRV_TO_ID: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn handle_log_start(
    obj: &Map<String, Value>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log start"))?;

    let text = obj
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Create new log buffer for this id
    if let Ok(mut buffers) = LOG_BUFFERS.lock() {
        buffers.insert(id, Vec::new());
    }

    // Map id to derivation name
    if let Ok(mut drv_to_id) = DRV_TO_ID.lock() {
        //println!("{}", text.clone());
        drv_to_id.insert(text.clone(), id);
    }

    if display.debug {
        println!("Log start: id={}, text='{}'", id, text);
    }

    Ok(())
}

pub async fn handle_log_line(
    obj: &Map<String, Value>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log line"))?;

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

    let message = NixMessage {
        action: "result".to_string(),
        message_type: Some(101), // resBuildLogLine
        content,
        level: None,
        file: None,
    };

    // Add to buffer
    if let Ok(mut buffers) = LOG_BUFFERS.lock() {
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.push(message);
        }
    }

    if display.debug {
        println!("Log line: id={}", id);
    }

    Ok(())
}

pub async fn handle_log_phase(
    obj: &Map<String, Value>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log phase"))?;

    let fields = obj.get("fields").and_then(|v| v.as_array());

    let content = if let Some(fields) = fields {
        if !fields.is_empty() {
            format!("Phase: {}", fields[0].as_str().unwrap_or(""))
        } else {
            "Phase: unknown".to_string()
        }
    } else {
        "Phase: unknown".to_string()
    };

    let message = NixMessage {
        action: "result".to_string(),
        message_type: Some(104), // resSetPhase
        content,
        level: None,
        file: None,
    };

    // Add to buffer
    if let Ok(mut buffers) = LOG_BUFFERS.lock() {
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.push(message);
        }
    }

    if display.debug {
        println!("Log phase: id={}", id);
    }

    Ok(())
}

pub async fn handle_log_stop(
    obj: &Map<String, Value>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log stop"))?;

    // if display.verbose {
    //     if let Ok(mut buffers) = LOG_BUFFERS.lock() {
    //         if let Some(buffer) = buffers.remove(&id) {
    //             display.print_log_buffer(id, &buffer).await;
    //         }
    //     }
    // }
    // // Just remove the buffer without printing
    // if let Ok(mut buffers) = LOG_BUFFERS.lock() {
    //     buffers.remove(&id);
    // }

    if display.debug {
        println!("Log stop: id={}", id);
    }

    Ok(())
}

pub async fn handle_msg(
    obj: &Map<String, Value>,
    display: &mut display::DisplayManager,
) -> Result<()> {
    let level = obj.get("level").and_then(|v| v.as_u64()).unwrap_or(0);
    let msg = obj.get("msg").and_then(|v| v.as_str()).unwrap_or("");
    let file = obj.get("file").and_then(|v| v.as_str());

    let re = Regex::new(r#"/nix/store/([a-zA-Z0-9_.+-]+).drv"#).unwrap();
    let captures = re.captures(msg);

    match captures {
        Some(c) => {
            if let Some(c) = c.get(1) {
                // lv24iib6cgsr1ipkz4gpf2agf08bxj6n-cargo-0_88_0-d76731b471aa2da9
                let drv: String = format!("building '/nix/store/{}.drv'", c.as_str());
                if let Ok(drv_to_id) = DRV_TO_ID.lock() {
                    match drv_to_id.get(&drv) {
                        Some(id) => {
                            //println!("{msg} {id} handle_msg");

                            if let Ok(mut buffers) = LOG_BUFFERS.lock() {
                                if let Some(buffer) = buffers.remove(&id) {
                                    display.print_log_buffer(*id, &buffer, c.as_str()).await;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }
        None => {}
    }

    // Filter by level based on mode
    let should_show = if display.verbose {
        true // Show all levels in verbose mode
    } else {
        level >= 1 && level <= 3 // Normal mode: show levels 1, 2, 3
    };

    if should_show {
        display.print_message(level, msg, file).await;
    }

    if display.debug {
        println!("Message: level={}, file={:?}, msg='{}'", level, file, msg);
    }

    Ok(())
}

pub fn has_log_buffer(id: u64) -> bool {
    if let Ok(buffers) = LOG_BUFFERS.lock() {
        buffers.contains_key(&id)
    } else {
        false
    }
}

pub fn query_logs_by_id(id: u64) -> Option<LogData> {
    if let Ok(buffers) = LOG_BUFFERS.lock() {
        buffers.get(&id).cloned()
    } else {
        None
    }
}
