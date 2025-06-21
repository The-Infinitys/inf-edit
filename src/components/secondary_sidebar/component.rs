use super::help_widget::HelpWidget;
use ratatui::{layout::Rect, Frame};

/// Enum to hold all possible components for the secondary sidebar.
pub enum SecondarySidebarComponent {
    Help(HelpWidget),
    // Other components can be added here.
}

impl SecondarySidebarComponent {
    /// A render method to dispatch drawing to the correct underlying component.
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            SecondarySidebarComponent::Help(h) => h.render(f, area),
        }
    }

    /// Helper to get a mutable reference to the HelpWidget if it exists.
    pub fn as_help_widget_mut(&mut self) -> Option<&mut HelpWidget> {
        match self {
            SecondarySidebarComponent::Help(h) => Some(h),
        }
    }
}