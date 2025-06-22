use super::help_widget::HelpWidget;
use crate::theme::Theme;
use ratatui::{layout::Rect, Frame};

pub enum SecondarySidebarComponent {
    Help(HelpWidget),
}

impl SecondarySidebarComponent {
    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        match self {
            SecondarySidebarComponent::Help(h) => h.render(f, area, theme),
        }
    }
}
