use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::process::Command;

#[derive(Default)]
pub struct GitWidget {
    status_output: Vec<Line<'static>>,
}

impl GitWidget {
    pub fn new() -> Self {
        let mut widget = Self::default();
        widget.update_status();
        widget
    }

    fn update_status(&mut self) {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let s = String::from_utf8_lossy(&output.stdout);
                self.status_output = s.lines().map(|line| {
                    let status_code = line[..2].to_string();
                    let path = line[3..].to_string();
                    let color = match status_code.as_str() {
                        "M " | "MM" => Color::Yellow, // Modified
                        "A " | "AM" => Color::Green,  // Added
                        "D " | "AD" => Color::Red,    // Deleted
                        "?? " => Color::DarkGray,     // Untracked
                        _ => Color::White,
                    };
                    Line::from(vec![
                        Span::styled(status_code, Style::default().fg(color)),
                        Span::raw(" "),
                        Span::raw(path),
                    ])
                }).collect();
            }
            _ => {
                self.status_output = vec![Line::from(Span::styled("Not a git repository or git not found.", Style::default().fg(Color::Red)))];
            }
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, is_active: bool) {
        let mut git_block = Block::default()
            .title("Git Status")
            .borders(Borders::ALL);
        if is_active {
            git_block = git_block.style(Style::default().fg(Color::Yellow));
        }
        let text = Paragraph::new(self.status_output.clone()).block(git_block);
        f.render_widget(text, area);
    }
}