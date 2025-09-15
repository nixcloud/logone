use crate::logone;
use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

// Status state variables
static mut DONE: u64 = 0;
static mut EXPECTED: u64 = 0;
static mut RUNNING: u64 = 0;
static mut FAILED: u64 = 0;
static STATUS_IDS: LazyLock<Mutex<HashSet<u64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn handle_status_start(obj: &Map<String, Value>, _: &mut logone::LogOne) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in stats start"))?;

    // Track this as a status ID
    if let Ok(mut status_ids) = STATUS_IDS.lock() {
        status_ids.insert(id);
    }
    Ok(())
}

pub async fn handle_status_update(
    obj: &Map<String, Value>,
    display: &mut logone::LogOne,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in stats update"))?;

    // Check if the id is in STATUS_IDS
    if let Ok(status_ids) = STATUS_IDS.lock() {
        if !status_ids.contains(&id) {
            return Err(anyhow!("Unknown id in stats update: {}", id));
        }
    } else {
        return Err(anyhow!("Failed to lock STATS_IDS"));
    }

    let fields = obj
        .get("fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing fields array in stats update"))?;

    if fields.len() != 4 {
        return Err(anyhow!(
            "Expected 4 fields in stats update, got {}",
            fields.len()
        ));
    }

    let done = fields[0].as_u64().unwrap_or(0);
    let expected = fields[1].as_u64().unwrap_or(0);
    let running = fields[2].as_u64().unwrap_or(0);
    let failed = fields[3].as_u64().unwrap_or(0);

    // Update stats variables (absolute values)
    unsafe {
        DONE = done;
        EXPECTED = expected;
        RUNNING = running;
        FAILED = failed;
    }

    update_stats_display(display).await?;

    Ok(())
}

pub async fn handle_status_stop(
    obj: &Map<String, Value>,
    display: &mut logone::LogOne,
) -> Result<()> {
    let id = obj
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Missing id in stats stop"))?;

    // Remove from stats IDs
    if let Ok(mut status_ids) = STATUS_IDS.lock() {
        status_ids.remove(&id);
    }

    // Keep the stats values, don't reset them
    update_stats_display(display).await?;

    Ok(())
}

pub fn is_status_id(id: u64) -> bool {
    if let Ok(status_ids) = STATUS_IDS.lock() {
        status_ids.contains(&id)
    } else {
        false
    }
}

async fn update_stats_display(display: &mut logone::LogOne) -> Result<()> {
    let (done, expected, running, failed) = unsafe { (DONE, EXPECTED, RUNNING, FAILED) };

    display.update_stats(done, expected, running, failed).await;
    Ok(())
}
