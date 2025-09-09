use serde_json::{Map, Value};
use anyhow::Result;
use crate::display;

pub async fn handle_misc(obj: &Map<String, Value>, display: &mut display::DisplayManager) -> Result<()> {
    // Empty handler for now - ignores all unhandled messages
    
    if display.debug {
        let action = obj.get("action").and_then(|v| v.as_str()).unwrap_or("unknown");
        let message_type = obj.get("type").and_then(|v| v.as_u64());
        println!("Misc handler: action={}, type={:?}", action, message_type);
    }
    
    Ok(())
}