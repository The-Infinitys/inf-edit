use std::{time::SystemTime, process::Command};
use chrono::{Local, DateTime};
use ratatui::{
    Frame,
    layout::{Rect},
    style::{Style, Color},
    text::{Line, Span},
    widgets::{Paragraph},
};


pub struct StatusBar {
    pub message: String,
}

impl StatusBar {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
    }

    fn get_git_info(&self) -> String {
        let output = Command::new("git")
            .args(&["status", "--porcelain", "--branch"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let s = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = s.lines().collect();
                let branch_line = lines.iter().find(|line| line.starts_with("##")).unwrap_or(&"");
                let branch = branch_line.split(" ").nth(1).unwrap_or("No branch info");
                let changes = lines.len().saturating_sub(1);
                format!("Git: {} ({})", branch, if changes > 0 { format!("{} changes", changes) } else { "clean".into() })
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

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let git_info = self.get_git_info();
        let current_time = self.get_current_time();
        let resource_usage = self.get_resource_usage();

        let status_line = Line::from(vec![
            Span::styled(self.message.clone(), Style::default()),
            Span::raw(" | "),
            Span::styled(git_info, Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::styled(current_time, Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(resource_usage, Style::default().fg(Color::Magenta)),
        ]);

        let paragraph = Paragraph::new(status_line);
        f.render_widget(paragraph, area); // Ensure this renders the paragraph
    }
}