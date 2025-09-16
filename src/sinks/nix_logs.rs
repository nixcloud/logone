use crate::logone;
use anyhow::{anyhow, Result};
use logone::{LogStatus, NixMessage};
use regex::Regex;
use serde_json::{Map, Value};

pub fn handle_log_start(obj: &Map<String, Value>, logone: &mut logone::LogOne) -> Result<()> {
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
    logone.nix_log_buffers.insert(id, Vec::new());
    logone.nix_log_buffers_state.insert(id, LogStatus::Started);

    // Map id to derivation name
    //println!("{}", text.clone());
    logone.drv_to_id.insert(text.clone(), id);

    Ok(())
}

pub fn handle_log_line(obj: &Map<String, Value>, logone: &mut logone::LogOne) -> Result<()> {
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
    if let Some(buffer) = logone.nix_log_buffers.get_mut(&id) {
        buffer.push(message);
    }

    Ok(())
}

pub fn handle_log_phase(obj: &Map<String, Value>, logone: &mut logone::LogOne) -> Result<()> {
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
    if let Some(buffer) = logone.nix_log_buffers.get_mut(&id) {
        buffer.push(message);
    }

    Ok(())
}

pub fn handle_log_stop(obj: &Map<String, Value>, logone: &mut logone::LogOne) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log phase"))?;
    logone.nix_log_buffers_state.insert(id, LogStatus::Stopped);
    Ok(())
}

pub fn handle_msg(obj: &Map<String, Value>, logone: &mut logone::LogOne) -> Result<()> {
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
                logone.print_log_buffer_by_drv(drv);
            }
        }
        None => {}
    }

    // Show messages with level 1-3 (WARN, NOTICE, INFO)
    if level >= 1 && level <= 3 {
        logone.print_message(level, msg, file);
    }

    Ok(())
}

pub fn has_log_buffer(id: u64, logone: &mut logone::LogOne) -> bool {
    logone.nix_log_buffers.contains_key(&id)
}

pub fn query_logs_by_id(id: u64, logone: &mut logone::LogOne) -> Option<Vec<NixMessage>> {
    logone.nix_log_buffers.get(&id).cloned()
}
