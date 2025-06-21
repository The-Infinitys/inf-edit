use ratatui::{
    prelude::*, // Keep for Span::styled
    text::Line,
    widgets::Paragraph,
};
use chrono::{DateTime, Local};
#[allow(unused_imports)] // Allow unused import for colored, as it's used for String extension methods
use colored::*;
use std::{process::Command, time::SystemTime};

pub trait BottomBarItem {
    fn render_item(&self, f: &mut Frame, area: Rect);
}

/// Renders a simple text string with a given alignment.
pub struct TextItem {
    text: String,
    alignment: Alignment,
}

impl TextItem {
    pub fn new(text: String, alignment: Alignment) -> Self {
        Self { text, alignment }
    }
}

impl BottomBarItem for TextItem {
    fn render_item(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(Line::from(self.text.clone())).alignment(self.alignment);
        f.render_widget(paragraph, area);
    }
}

/// Renders Git status information.
pub struct GitInfoItem {
    // No fields needed, as it fetches data dynamically
}

impl Default for GitInfoItem {
    fn default() -> Self {
        Self::new()
    }
}

impl GitInfoItem {
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
}

impl BottomBarItem for GitInfoItem {
    fn render_item(&self, f: &mut Frame, area: Rect) {
        let text = self.get_git_info().red().to_string();
        let paragraph = Paragraph::new(Line::from(text)).alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }
}

/// Renders the current time.
pub struct CurrentTimeItem {}

impl Default for CurrentTimeItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentTimeItem {
    pub fn new() -> Self { Self {} }
    fn get_current_time(&self) -> String {
        let now: DateTime<Local> = SystemTime::now().into();
        now.format("Time: %Y-%m-%d %H:%M:%S").to_string()
    }
}

impl BottomBarItem for CurrentTimeItem {
    fn render_item(&self, f: &mut Frame, area: Rect) {
        let text = self.get_current_time().yellow().to_string();
        let paragraph = Paragraph::new(Line::from(text)).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Renders resource usage (placeholder).
pub struct ResourceUsageItem {}

impl Default for ResourceUsageItem {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceUsageItem {
    pub fn new() -> Self { Self {} }
    fn get_resource_usage(&self) -> String {
        "CPU: 25% Mem: 60%".to_string() // Placeholder
    }
}

impl BottomBarItem for ResourceUsageItem {
    fn render_item(&self, f: &mut Frame, area: Rect) {
        let text = self.get_resource_usage().green().to_string();
        let paragraph = Paragraph::new(Line::from(text)).alignment(Alignment::Right);
        f.render_widget(paragraph, area);
    }
}