use anyhow::Result;
use clap::ValueEnum;
use console::style;
use crossterm::{
    cursor::{MoveToColumn, MoveToPreviousLine},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{stdout, Write};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, ValueEnum, Eq, PartialEq)]
pub enum LogLevel {
    Cargo,
    Errors,
    Verbose,
}

#[derive(Debug, Clone, Copy, ValueEnum, Eq, PartialEq)]
pub enum LogStatus {
    Started,
    Stopped,
    FinishedWithSuccess,
    FinishedWithError,
}

#[derive(Debug, Clone)]
pub struct NixMessage {
    pub action: String,
    pub message_type: Option<u64>,
    pub content: String,
    pub level: Option<u64>,
    pub file: Option<String>,
}

pub type Id = u64;

pub struct LogOne {
    pub colored: bool,
    log_level: LogLevel,
    status_line_active: bool,
    targets: HashMap<String, u64>,
    last_stats: Option<(u64, u64, u64, u64)>,
    last_targets_hash: u64,
    pub nix_log_buffers: HashMap<Id, Vec<NixMessage>>,
    pub nix_log_buffers_state: HashMap<Id, LogStatus>,
    pub cargo_log_buffers: HashMap<Id, Vec<String>>,
    pub cargo_log_buffers_state: HashMap<Id, LogStatus>,
    pub drv_to_id: HashMap<String, u64>,
    active: bool,
}

impl Drop for LogOne {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl LogOne {
    pub fn new(colored: bool, log_level: LogLevel) -> Self {
        Self {
            colored,
            log_level,
            status_line_active: false,
            targets: HashMap::new(),
            last_stats: None,
            last_targets_hash: 0,
            nix_log_buffers: HashMap::new(),
            nix_log_buffers_state: HashMap::new(),
            cargo_log_buffers: HashMap::new(),
            cargo_log_buffers_state: HashMap::new(),
            drv_to_id: HashMap::new(),
            active: true,
        }
    }

    pub fn shutdown(&mut self) {
        if self.active {
            self.active = false;
            let ids: Vec<u64> = self.nix_log_buffers.keys().cloned().collect();
            if self.level() == LogLevel::Verbose {
                for id in ids {
                    self.print_log_buffer_by_id(id);
                }
            }
        }
    }

    pub fn level(&self) -> LogLevel {
        self.log_level
    }

    // Snapshot of targets with their counts
    fn snapshot_targets(&self) -> Vec<(String, u64)> {
        let mut snapshot: Vec<(String, u64)> = self
            .targets
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|(name, &count)| (name.clone(), count))
            .collect();
        snapshot.sort_by(|a, b| a.0.cmp(&b.0)); // Sort alphabetically by name
        snapshot
    }

    pub fn target_add(&mut self, create_name: String) -> Result<()> {
        self.targets
            .entry(create_name)
            .and_modify(|c| *c += 1)
            .or_insert(1);
        Ok(())
    }

    pub fn target_remove(&mut self, create_name: String) -> Result<()> {
        if let Some(count) = self.targets.get_mut(&create_name) {
            *count -= 1;
            if *count == 0 {
                self.targets.remove(&create_name);
            }
        }
        // If target doesn't exist, ignore (no-op)
        Ok(())
    }

    pub fn clear_status(&mut self) {
        if self.status_line_active {
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
            self.status_line_active = false;
        }
    }

    // Draw status line with current stored values (ignores anti-flicker logic)
    pub fn draw_status(&mut self) {
        // Only redraw if we have stats to show
        if let Some((done, expected, running, failed)) = self.last_stats {
            let base_status = format!(
                "[ {} Done | {} Expected | {} Running | {} Failed ]",
                done, expected, running, failed
            );

            // Get snapshot for consistent display
            let targets_snapshot = self.snapshot_targets();
            let display_line = self.format_status_with_targets(&base_status, &targets_snapshot);

            if self.colored {
                let styled_base = format!(
                    "[ {} Done | {} Expected | {} Running | {} Failed ]",
                    style(done).green(),
                    style(expected).green(),
                    style(running).yellow(),
                    style(failed).red()
                );

                // Get targets part and combine with styled base
                let targets_part = self.get_targets_display(&base_status, &targets_snapshot);
                if targets_part.is_empty() {
                    println!("{}", styled_base);
                } else {
                    println!("{} {}", styled_base, targets_part);
                }
            } else {
                println!("{}", display_line);
            }

            self.status_line_active = true;
            stdout().flush().unwrap();
        }
    }

    pub fn update_stats(&mut self, done: u64, expected: u64, running: u64, failed: u64) {
        let current_stats = (done, expected, running, failed);
        // Get snapshot for consistent hash and display
        let targets_snapshot = self.snapshot_targets();
        let current_targets_hash = self.calculate_targets_hash(&targets_snapshot);

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
        let display_line = self.format_status_with_targets(&base_status, &targets_snapshot);

        if self.colored {
            let styled_base = format!(
                "[ {} Done | {} Expected | {} Running | {} Failed ]",
                style(done).green(),
                style(expected).green(),
                style(running).yellow(),
                style(failed).red()
            );

            // Get targets part and combine with styled base
            let targets_part = self.get_targets_display(&base_status, &targets_snapshot);
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

    pub fn print_log_buffer_by_drv(&mut self, drv: String) {
        let id: Id = match self.drv_to_id.get(&drv) {
            Some(id) => *id,
            None => return,
        };
        self.print_log_buffer(id, drv)
    }

    pub fn print_log_buffer_by_id(&mut self, id: Id) {
        let drv: String = match self.drv_to_id.iter().find(|(_, &v)| v == id) {
            Some((k, _)) => k.clone(),
            None => return,
        };
        self.print_log_buffer(id, drv)
    }

    fn print_log_buffer(&mut self, id: Id, drv: String) {
        // Extract the buffer
        let buffer = self.nix_log_buffers.remove(&id);

        if let Some(buffer) = buffer {
            // Clear status line if active
            self.clear_status();

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

            // Redraw status line after printing log buffer
            self.draw_status();
        }
    }

    pub fn print_message(&mut self, level: u64, msg: &str, file: Option<&str>) {
        // Clear status line if active
        if self.status_line_active {
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
            self.status_line_active = false;
        }

        let formatted_msg = if let Some(file_path) = file {
            format!("{}: {}", file_path, msg)
        } else {
            format!("{}", msg)
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

        // Redraw status line after printing message
        self.draw_status();
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

    fn format_status_with_targets(
        &self,
        base_status: &str,
        targets_snapshot: &[(String, u64)],
    ) -> String {
        let targets_display = self.get_targets_display(base_status, targets_snapshot);
        if targets_display.is_empty() {
            base_status.to_string()
        } else {
            format!("{} {}", base_status, targets_display)
        }
    }

    fn get_targets_display(&self, base_status: &str, targets_snapshot: &[(String, u64)]) -> String {
        if targets_snapshot.is_empty() {
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

        for (name, count) in targets_snapshot {
            let target_display = if *count > 1 {
                format!("{} (×{})", name, count)
            } else {
                name.clone()
            };

            let addition = if first {
                target_display
            } else {
                format!(", {}", target_display)
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

    fn calculate_targets_hash(&self, targets_snapshot: &[(String, u64)]) -> u64 {
        let mut hasher = DefaultHasher::new();
        targets_snapshot.hash(&mut hasher);
        hasher.finish()
    }
}
