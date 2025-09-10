use crate::LogLevel;
use anyhow::Result;
use console::style;
use crossterm::{
    cursor::{MoveToColumn, MoveToPreviousLine, RestorePosition, SavePosition},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{stdout, Write};

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
pub type Id = u64;

pub struct DisplayManager {
    pub colored: bool,
    log_level: LogLevel,
    status_line_active: bool,
    targets: Vec<String>,
    // State tracking to prevent unnecessary redraws
    last_stats: Option<(u64, u64, u64, u64)>,
    last_targets_hash: u64,
    pub log_buffers: LazyLock<Mutex<HashMap<Id, LogData>>>,
    pub drv_to_id: LazyLock<Mutex<HashMap<String, u64>>>,
}

impl DisplayManager {
    pub fn new(colored: bool, log_level: LogLevel) -> Self {
        Self {
            colored,
            log_level,
            status_line_active: false,
            targets: Vec::new(),
            last_stats: None,
            last_targets_hash: 0,
            log_buffers: LazyLock::new(|| Mutex::new(HashMap::new())),
            drv_to_id: LazyLock::new(|| Mutex::new(HashMap::new())),
        }
    }

    pub fn level(&self) -> LogLevel {
        self.log_level
    }
    pub async fn target_add(&mut self, create_name: String) -> Result<()> {
        self.targets.push(create_name);
        self.last_targets_hash = self.calculate_targets_hash();
        Ok(())
    }

    pub async fn target_remove(&mut self, create_name: String) -> Result<()> {
        if let Some(pos) = self.targets.iter().position(|x| *x == create_name) {
            self.targets.remove(pos);
        }
        self.last_targets_hash = self.calculate_targets_hash();
        Ok(())
    }

    pub async fn update_stats(&mut self, done: u64, expected: u64, running: u64, failed: u64) {
        let current_stats = (done, expected, running, failed);
        let current_targets_hash = self.calculate_targets_hash();

        // Check if anything actually changed
        let stats_changed = self.last_stats.as_ref() != Some(&current_stats);
        let targets_changed = self.last_targets_hash != current_targets_hash;

        if !stats_changed && !targets_changed {
            return; // Nothing changed, no need to redraw
        }

        if self.status_line_active {
            // Move cursor up to rewrite the status line
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
        }

        let base_status = format!(
            "[ {} Done | {} Expected | {} Running | {} Failed ]",
            done, expected, running, failed
        );

        // Get terminal width and crop targets to fit
        let display_line = self.format_status_with_targets(&base_status);

        if self.colored {
            let styled_base = format!(
                "[ {} Done | {} Expected | {} Running | {} Failed ]",
                style(done).green(),
                style(expected).green(),
                style(running).yellow(),
                style(failed).red()
            );

            // Get targets part and combine with styled base
            let targets_part = self.get_targets_display(&base_status);
            if targets_part.is_empty() {
                println!("{}", styled_base);
            } else {
                println!("{} {}", styled_base, targets_part);
            }
        } else {
            println!("{}", display_line);
        }

        self.status_line_active = true;
        self.last_stats = Some(current_stats);
        self.last_targets_hash = current_targets_hash;
        stdout().flush().unwrap();
    }

    pub async fn print_log_buffer_by_drv(&mut self, drv: String) {
        let id: Id = {
            let drv_to_id = match self.drv_to_id.lock() {
                Ok(guard) => guard,
                Err(_) => return, // bail out if the lock is poisoned
            };
            match drv_to_id.get(&drv) {
                Some(id) => *id, // extract the value
                None => return,  // early exit if not found
            }
        };
        self.print_log_buffer(id, drv).await
    }

    pub async fn print_log_buffer_by_id(&mut self, id: Id) {
        let drv: String = match self.drv_to_id.lock() {
            Ok(drv_to_id) => match drv_to_id.iter().find(|(_, &v)| v == id) {
                Some((k, _)) => k.clone(),
                None => return,
            },
            Err(_) => return,
        };

        self.print_log_buffer(id, drv).await
    }

    async fn print_log_buffer(&mut self, id: Id, drv: String) {
        if let Ok(mut buffers) = self.log_buffers.lock() {
            if let Some(buffer) = buffers.remove(&id) {
                // Clear status line if active
                if self.status_line_active {
                    let mut stdout = stdout();
                    stdout.execute(MoveToPreviousLine(1)).unwrap();
                    stdout.execute(MoveToColumn(0)).unwrap();
                    stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                    self.status_line_active = false;
                }

                println!("Build log for '{}':", drv);
                for message in buffer {
                    match message.message_type {
                        Some(101) => {
                            // resBuildLogLine
                            if self.colored {
                                println!("  {}", style(&message.content).dim());
                            } else {
                                println!("  {}", message.content);
                            }
                        }
                        Some(104) => {
                            // resSetPhase
                            if self.colored {
                                println!("  {}", style(&message.content).cyan());
                            } else {
                                println!("  {}", message.content);
                            }
                        }
                        _ => {
                            println!("  {}", message.content);
                        }
                    }
                }
                println!(); // Empty line after log
                stdout().flush().unwrap();
            }
        }
    }

    pub async fn print_message(&mut self, level: u64, msg: &str, file: Option<&str>) {
        // Clear status line if active
        if self.status_line_active {
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
            self.status_line_active = false;
        }

        let level_text = match level {
            0 => "ERROR",
            1 => "WARN",
            2 => "NOTICE",
            3 => "INFO",
            _ => "DEBUG",
        };

        let formatted_msg = if let Some(file_path) = file {
            format!("[{}] {}: {}", level_text, file_path, msg)
        } else {
            format!("[{}] {}", level_text, msg)
        };

        if self.colored {
            let styled_msg = match level {
                0 => style(formatted_msg).red(),
                1 => style(formatted_msg).yellow(),
                2 => style(formatted_msg).blue(),
                3 => style(formatted_msg).green(),
                _ => style(formatted_msg).dim(),
            };
            println!("{}", styled_msg);
        } else {
            println!("{}", formatted_msg);
        }

        stdout().flush().unwrap();
    }

    pub fn clear_status_line(&mut self) {
        if self.status_line_active {
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
            self.status_line_active = false;
            stdout.flush().unwrap();
        }
    }

    fn format_status_with_targets(&self, base_status: &str) -> String {
        let targets_display = self.get_targets_display(base_status);
        if targets_display.is_empty() {
            base_status.to_string()
        } else {
            format!("{} {}", base_status, targets_display)
        }
    }

    fn get_targets_display(&self, base_status: &str) -> String {
        if self.targets.is_empty() {
            return String::new();
        }

        // Get terminal width, default to 80 if unable to detect
        let terminal_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);

        let base_len = base_status.len();
        let space_for_targets = if terminal_width > base_len + 1 {
            terminal_width - base_len - 1 // -1 for the space between base and targets
        } else {
            return String::new(); // No space for targets
        };

        // Build comma-separated list, cropping if necessary
        let mut result = String::new();
        let mut first = true;

        for target in &self.targets {
            let addition = if first {
                target.clone()
            } else {
                format!(", {}", target)
            };

            if result.len() + addition.len() <= space_for_targets {
                result.push_str(&addition);
                first = false;
            } else {
                // If we can't fit this target, stop here
                break;
            }
        }

        result
    }

    fn calculate_targets_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.targets.hash(&mut hasher);
        hasher.finish()
    }
}
