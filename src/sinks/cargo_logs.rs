use crate::{logone, logone::LogStatus};
use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

// echo "@cargo { \"type\":0, \"crate_name\":\"{{{crate_name}}}\", \"id\":\"{{{fullname}}}\" }"
pub async fn handle_cargo_log_start(
    obj: &Map<String, Value>,
    logone: &mut logone::LogOne,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log start"))?;

    // Create new log buffer for this id
    if let Ok(mut buffers) = logone.cargo_log_buffers.lock() {
        buffers.insert(id, Vec::new());
    }

    if let Ok(mut buffers_state) = logone.cargo_log_buffers_state.lock() {
        buffers_state.insert(id, LogStatus::Started);
    }

    let crate_name = obj
        .get("crate_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    logone.target_add(crate_name).await?;
    Ok(())
}

// @cargo {type: 2, id: $fullname, crate_name: $crate_name, rustc_exit_code: ($exit_code|tonumber), rustc_messages: [ some embedded rustc output messages]}
pub async fn handle_cargo_log_exit(
    obj: &Map<String, Value>,
    logone: &mut logone::LogOne,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in log phase"))?;

    let crate_name = obj
        .get("crate_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    logone.target_remove(crate_name).await?;

    let rustc_exit_code: u64 = obj.get("rustc_exit_code").and_then(|v| v.as_u64()).unwrap();

    let rendered_messages: Vec<String> = obj
        .get("rustc_messages")
        .and_then(|msgs| msgs.as_array())
        .map(|msgs| {
            msgs.iter()
                .filter_map(|msg| {
                    msg.as_object()
                        .and_then(|obj| obj.get("rendered"))
                        .and_then(|value| value.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);

    if let Ok(mut buffers_state) = logone.cargo_log_buffers_state.lock() {
        buffers_state.insert(id, LogStatus::Started);
        match rustc_exit_code {
            0 => {
                buffers_state.insert(id, LogStatus::FinishedWithSuccess);
            }
            1 => {
                buffers_state.insert(id, LogStatus::FinishedWithError);
            }
            _ => {
                buffers_state.insert(id, LogStatus::Stopped);
            }
        };
    }

    if let Ok(mut buffers) = logone.cargo_log_buffers.lock() {
        if let Some(buffer) = buffers.get_mut(&id) {
            for msg in rendered_messages.clone() {
                buffer.push(msg);
            }
        }
    }

    for msg in rendered_messages {
        println!("{}", msg);
    }

    Ok(())
}
