use crate::logs::{LogData, NixMessage};
use console::{style, Color, Style};
use crossterm::{
    cursor::{MoveToColumn, MoveToPreviousLine, RestorePosition, SavePosition},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};

pub struct DisplayManager {
    pub verbose: bool,
    pub colored: bool,
    pub timing: bool,
    pub debug: bool,
    status_line_active: bool,
}

impl DisplayManager {
    pub fn new(verbose: bool, colored: bool, timing: bool, debug: bool) -> Self {
        Self {
            verbose,
            colored,
            timing,
            debug,
            status_line_active: false,
        }
    }

    pub async fn update_stats(&mut self, done: u64, expected: u64, running: u64, failed: u64) {
        // Introduce a delay of some ms for interactive testing with a fixed log
        // FIXME remove this later
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

        if self.status_line_active {
            // Move cursor up to rewrite the status line
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
        }

        let status_text = format!(
            "[ {} Done | {} Expected | {} Running | {} Failed ]",
            done, expected, running, failed
        );

        if self.colored {
            let styled_text = format!(
                "[ {} Done | {} Expected | {} Running | {} Failed ]",
                style(done).green(),
                style(expected).green(),
                style(running).yellow(),
                style(failed).red()
            );
            println!("{}", styled_text);
        } else {
            println!("{}", status_text);
        }

        self.status_line_active = true;
        stdout().flush().unwrap();
    }

    pub async fn print_log_buffer(&mut self, id: u64, buffer: &LogData, name: &str) {
        if buffer.is_empty() {
            return;
        }

        // Clear status line if active
        if self.status_line_active {
            let mut stdout = stdout();
            stdout.execute(MoveToPreviousLine(1)).unwrap();
            stdout.execute(MoveToColumn(0)).unwrap();
            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
            self.status_line_active = false;
        }

        println!("Build log for '{}':", name);
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
}
