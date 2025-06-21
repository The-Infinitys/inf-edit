use chrono::{DateTime, Local};
use colored::*;
use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};
use std::{process::Command, time::SystemTime};
pub struct BottomBar {}

impl Default for BottomBar {
    fn default() -> Self {
        Self::new()
    }
}

impl BottomBar {
    pub fn new() -> Self {
        Self {}
    }

    fn get_git_info(&self) -> String {
        let output = Command::new("git")
            .args(["status", "--porcelain", "--branch"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let s = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = s.lines().collect();
                let branch_line = lines
                    .iter()
                    .find(|line| line.starts_with("##"))
                    .unwrap_or(&"");
                let branch = branch_line.split(" ").nth(1).unwrap_or("No branch info");
                let changes = lines.len().saturating_sub(1);
                format!(
                    "Git: {} ({})",
                    branch,
                    if changes > 0 {
                        format!("{} changes", changes)
                    } else {
                        "clean".into()
                    }
                )
            }
            _ => "Git: Not a git repository or git not found".to_string(),
        }
    }

    fn get_current_time(&self) -> String {
        let now: DateTime<Local> = SystemTime::now().into();
        now.format("Time: %Y-%m-%d %H:%M:%S").to_string()
    }

    fn get_resource_usage(&self) -> String {
        // This is a placeholder. In a real application, you'd use a crate like `sysinfo`
        // to get actual CPU and memory usage.

        "CPU: 25% Mem: 60%".to_string()
    }

    pub fn render(&self, f: &mut Frame, area: Rect, _is_active: bool) {
        // Added _is_active for component consistency
        let git_info = self.get_git_info();
        let current_time = self.get_current_time();
        let resource_usage = self.get_resource_usage();
        let formatted_line = format!(
            "{:<0} | {:^0} | {:>0}",
            git_info.red(),
            current_time.yellow(),
            resource_usage.green()
        );
        let status_line = Line::from(formatted_line);

        let paragraph = Paragraph::new(status_line);
        f.render_widget(paragraph, area); // Ensure this renders the paragraph
    }
}
